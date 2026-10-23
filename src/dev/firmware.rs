//! The firmware pin map served at `GET /api/firmware`.

use std::path::Path;

use serde_json::{json, Value};
use tiny_http::Request;

use super::api::respond_json;
use super::server::query_param;
use super::state::State;
use crate::codegen::{categorize_pins, extract_mcu_pins, field_names, McuInfo};
use crate::config::{FirmwareConfig, FusibleConfig};
use crate::linter::{LintContext, PinMuxRule, Rule};
use crate::netlist::Netlist;
use crate::parser;

/// A status code and JSON body for the response.
pub(super) type Reply = (u16, String);

/// The `pin_mux` conflicts of a circuit as `(message, help)`.
type Conflicts = Vec<(String, Option<String>)>;

fn failure(status: u16, message: String) -> Reply {
    (status, json!({ "error": message }).to_string())
}

/// Serves the pin map of the firmware named by `?name=`.
pub(super) fn serve(request: Request, state: &State) {
    let name = query_param(request.url(), "name").unwrap_or_default();
    let (path, cfg) = FusibleConfig::nearest().unzip();
    let root = path
        .as_deref()
        .and_then(Path::parent)
        .unwrap_or(Path::new(""));
    let (status, body) = firmware_reply(cfg.as_ref(), state, &name, root);
    respond_json(request, status, body);
}

/// Builds the pin map of firmware `name`, from the circuit of the board it runs on.
///
/// Paths are shown relative to `root`, the directory of the manifest.
pub(super) fn firmware_reply(
    cfg: Option<&FusibleConfig>,
    state: &State,
    name: &str,
    root: &Path,
) -> Reply {
    let Some((fw, board)) = cfg.and_then(|c| locate(c, name)) else {
        return failure(404, format!("unknown firmware `{name}`"));
    };
    let Some(slot) = state.slot(board.as_deref()) else {
        return failure(404, "the board of this firmware is not served".into());
    };
    let Some(source) = slot.source() else {
        return failure(503, "the board has not loaded yet".into());
    };
    let mcu = board
        .as_deref()
        .and_then(|b| cfg?.boards.get(b))
        .and_then(|b| b.mcu.clone());
    match pin_map(&source, mcu.as_deref()) {
        Ok(map) => (
            200,
            describe(name, fw, board.as_deref(), root, map).to_string(),
        ),
        Err(reason) => failure(422, with_hint(&reason, board.as_deref())),
    }
}

fn with_hint(reason: &str, board: Option<&str>) -> String {
    match board {
        Some(board) => format!("{reason}; set `mcu:` on board `{board}` in fusible.yml"),
        None => format!("{reason}; set `mcu.chip` in fusible.yml"),
    }
}

/// The firmware and the name of its board, which may be omitted with a single board.
fn locate<'a>(cfg: &'a FusibleConfig, name: &str) -> Option<(&'a FirmwareConfig, Option<String>)> {
    let fw = cfg.firmware.get(name)?;
    let only = match cfg.boards.len() {
        1 => cfg.boards.keys().next().cloned(),
        _ => None,
    };
    Some((fw, fw.board.clone().or(only)))
}

/// The resolved MCU and the `pin_mux` conflicts of the circuit.
struct PinMap {
    info: McuInfo,
    conflicts: Conflicts,
}

fn pin_map(source: &str, mcu: Option<&str>) -> Result<PinMap, String> {
    let circuit = parser::parse(source, Path::new("circuit.json")).map_err(|e| e.to_string())?;
    let netlist = Netlist::from_circuit(&circuit);
    let info = extract_mcu_pins(&circuit, &netlist, mcu).ok_or("no MCU found on the board")?;
    let ctx = LintContext::new(&circuit, &netlist).with_mcu(mcu);
    let conflicts = PinMuxRule
        .check(&ctx)
        .into_iter()
        .map(|d| (d.message, d.help))
        .collect();
    Ok(PinMap { info, conflicts })
}

fn describe(
    name: &str,
    fw: &FirmwareConfig,
    board: Option<&str>,
    root: &Path,
    map: PinMap,
) -> Value {
    let pins_rs = fw.pins_rs.as_deref();
    let violations: Vec<Value> = map
        .conflicts
        .iter()
        .map(|(message, help)| json!({ "message": message, "help": help }))
        .collect();
    json!({
        "name": name,
        "board": board,
        "mcu": map.info.chip,
        "component": map.info.name,
        "framework": fw.framework,
        "buildable": fw.build_cmd.is_some(),
        "pins_rs": pins_rs.map(|p| p.strip_prefix(root).unwrap_or(p)),
        "generated": pins_rs.is_some_and(Path::is_file),
        "pins": pin_items(&map.info, &map.conflicts),
        "violations": violations,
    })
}

fn pin_items(info: &McuInfo, conflicts: &Conflicts) -> Vec<Value> {
    let mut items = Vec::new();
    for (group, pins) in categorize_pins(&info.pins) {
        for (pin, field) in pins.iter().zip(field_names(&pins, &group)) {
            let quoted = format!("\"{}\"", pin.pin_label);
            let violation = conflicts.iter().find(|(m, _)| m.contains(&quoted));
            let remote = pin
                .remote_component
                .as_ref()
                .map(|c| match &pin.remote_port {
                    Some(port) => format!("{c}.{port}"),
                    None => c.clone(),
                });
            items.push(json!({
                "pin": pin.pin_label,
                "number": pin.pin_number,
                "group": group,
                "net": pin.net_name,
                "remote": remote,
                "const": format!("gpio::{}", pin.signal_name),
                "field": format!("{}_PINS.{field}", group.to_ascii_uppercase()),
                "violation": violation.map(|(message, _)| message),
            }));
        }
    }
    items
}
