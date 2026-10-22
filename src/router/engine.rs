use std::collections::BTreeMap;

use serde_json::{json, Value};

use super::astar::{find_path, AStarConfig, RoutePath, RouteStepKind, Terminal};
use super::spatial::{SpatialObstacle, SpatialRoutingIndex};
use crate::gerber::layers::PcbBoardElm;

#[derive(Clone, Debug)]
pub struct RouteOptions {
    pub clearance: f64,
    pub trace_width: f64,
    pub layers: u8,
    pub max_iterations: usize,
    pub dry_run: bool,
}

impl Default for RouteOptions {
    fn default() -> Self {
        Self {
            clearance: 0.15,
            trace_width: 0.2,
            layers: 2,
            max_iterations: 10,
            dry_run: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ConnectionJob {
    pub net_id: String,
    pub start: Terminal,
    pub target: Terminal,
}

#[derive(Debug)]
pub struct RouteResult {
    pub total_nets: usize,
    pub routed_nets: usize,
    pub unrouted_nets: Vec<String>,
    pub generated_traces: usize,
    pub generated_vias: usize,
    pub output_json: String,
}

/// Union-find over `source_port` and `source_net` ids, built from `source_trace` connectivity.
#[derive(Default)]
struct Connectivity {
    parent: BTreeMap<String, String>,
}

impl Connectivity {
    fn from_elements(elements: &[Value]) -> Self {
        let mut conn = Self::default();
        for el in elements.iter().filter(|el| type_of(el) == "source_trace") {
            let ids: Vec<String> = ["connected_source_port_ids", "connected_source_net_ids"]
                .iter()
                .filter_map(|key| el.get(*key).and_then(Value::as_array))
                .flatten()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect();
            for pair in ids.windows(2) {
                conn.union(&pair[0], &pair[1]);
            }
            if let [only] = ids.as_slice() {
                conn.find(only);
            }
        }
        conn
    }

    fn find(&mut self, id: &str) -> String {
        let mut root = id.to_string();
        while let Some(parent) = self.parent.get(&root).filter(|p| **p != root) {
            root = parent.clone();
        }
        let mut cur = id.to_string();
        while cur != root {
            let next = self
                .parent
                .insert(cur, root.clone())
                .unwrap_or_else(|| root.clone());
            cur = next;
        }
        self.parent
            .entry(root.clone())
            .or_insert_with(|| root.clone());
        root
    }

    fn union(&mut self, a: &str, b: &str) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra != rb {
            let (keep, drop) = if ra < rb { (ra, rb) } else { (rb, ra) };
            self.parent.insert(drop, keep);
        }
    }

    fn net_of(&mut self, id: &str) -> Option<String> {
        self.parent.contains_key(id).then(|| self.find(id))
    }
}

pub fn route_circuit(circuit_json: &str, options: &RouteOptions) -> Result<RouteResult, String> {
    let mut elements: Vec<Value> =
        serde_json::from_str(circuit_json).map_err(|e| format!("invalid Circuit JSON: {e}"))?;

    let config = AStarConfig {
        trace_width: options.trace_width,
        clearance: options.clearance,
        layers: if options.layers > 1 {
            vec!["top".to_string(), "bottom".to_string()]
        } else {
            vec!["top".to_string()]
        },
        ..Default::default()
    };

    let (base, net_terminals) = build_index(&elements, &config)?;

    let mut jobs = Vec::new();
    for (net_id, terminals) in &net_terminals {
        for pair in terminals.windows(2) {
            jobs.push(ConnectionJob {
                net_id: net_id.clone(),
                start: pair[0].clone(),
                target: pair[1].clone(),
            });
        }
    }

    let solved = route_jobs(&jobs, &base, &config, options.max_iterations);

    let mut routed_nets = 0;
    let mut unrouted_nets = Vec::new();
    let mut generated_vias = 0;
    let mut new_elements = Vec::new();

    for (job, maybe_path) in jobs.iter().zip(solved) {
        let Some(path) = maybe_path else {
            unrouted_nets.push(job.net_id.clone());
            continue;
        };
        routed_nets += 1;
        let route_array: Vec<Value> = path
            .steps
            .iter()
            .map(|step| match step {
                RouteStepKind::Wire { layer, x, y } => json!({
                    "route_type": "wire",
                    "x": x,
                    "y": y,
                    "width": options.trace_width,
                    "layer": layer
                }),
                RouteStepKind::Via {
                    x,
                    y,
                    from_layer,
                    to_layer,
                } => {
                    generated_vias += 1;
                    json!({
                        "route_type": "via",
                        "x": x,
                        "y": y,
                        "from_layer": from_layer,
                        "to_layer": to_layer,
                        "outer_diameter": config.via_diameter,
                        "hole_diameter": config.via_hole_diameter
                    })
                }
            })
            .collect();
        new_elements.push(json!({
            "type": "pcb_trace",
            "pcb_trace_id": format!("pcb_trace_auto_{routed_nets}"),
            "net_id": job.net_id,
            "route": route_array
        }));
    }

    if !options.dry_run {
        elements.extend(new_elements);
    }

    let output_json = serde_json::to_string_pretty(&elements)
        .map_err(|e| format!("failed to serialize routed JSON: {e}"))?;

    Ok(RouteResult {
        total_nets: jobs.len(),
        routed_nets,
        unrouted_nets,
        generated_traces: routed_nets,
        generated_vias,
        output_json,
    })
}

/// Routes jobs one at a time so every finished path becomes an obstacle; failed jobs move to the front for the next of up to `max_iterations` passes, and the pass routing the most jobs wins.
fn route_jobs(
    jobs: &[ConnectionJob],
    base: &SpatialRoutingIndex,
    config: &AStarConfig,
    max_iterations: usize,
) -> Vec<Option<RoutePath>> {
    let mut order: Vec<usize> = (0..jobs.len()).collect();
    let mut best: Vec<Option<RoutePath>> = vec![None; jobs.len()];
    let mut best_count = None;

    for _ in 0..max_iterations.max(1) {
        let mut index = base.clone();
        let mut paths: Vec<Option<RoutePath>> = vec![None; jobs.len()];
        for &j in &order {
            let job = &jobs[j];
            if let Some(path) = find_path(&job.start, &job.target, &job.net_id, &index, config) {
                insert_path(&mut index, &job.net_id, &path, config);
                paths[j] = Some(path);
            }
        }
        let routed = paths.iter().filter(|p| p.is_some()).count();
        if best_count.is_none_or(|b| routed > b) {
            best_count = Some(routed);
            best = paths.clone();
        }
        if routed == jobs.len() {
            break;
        }
        let (failed, done): (Vec<usize>, Vec<usize>) =
            order.iter().partition(|&&j| paths[j].is_none());
        order = failed.into_iter().chain(done).collect();
    }
    best
}

fn insert_path(
    index: &mut SpatialRoutingIndex,
    net_id: &str,
    path: &RoutePath,
    config: &AStarConfig,
) {
    let net = Some(net_id.to_string());
    for pair in path.steps.windows(2) {
        if let [RouteStepKind::Wire { layer, x, y }, RouteStepKind::Wire {
            layer: l2,
            x: x2,
            y: y2,
        }] = pair
        {
            if layer == l2 {
                index.insert_obstacle(
                    layer,
                    SpatialObstacle::trace(net.clone(), [*x, *y], [*x2, *y2], config.trace_width),
                );
            }
        }
    }
    for step in &path.steps {
        if let RouteStepKind::Via { x, y, .. } = step {
            for layer in &config.layers {
                index.insert_obstacle(
                    layer,
                    SpatialObstacle::via(net.clone(), [*x, *y], config.via_diameter),
                );
            }
        }
    }
}

type NetTerminals = BTreeMap<String, Vec<Terminal>>;

/// Builds the obstacle index (board outline, pads, plated and bare holes, vias, existing traces) and groups pad terminals by net.
fn build_index(
    elements: &[Value],
    config: &AStarConfig,
) -> Result<(SpatialRoutingIndex, NetTerminals), String> {
    let mut conn = Connectivity::from_elements(elements);
    let port_to_source: BTreeMap<&str, &str> = elements
        .iter()
        .filter(|el| type_of(el) == "pcb_port")
        .filter_map(|el| Some((str_of(el, "pcb_port_id")?, str_of(el, "source_port_id")?)))
        .collect();
    let trace_net: BTreeMap<String, String> = elements
        .iter()
        .filter(|el| type_of(el) == "source_trace")
        .filter_map(|el| {
            let id = str_of(el, "source_trace_id")?;
            let member = ["connected_source_port_ids", "connected_source_net_ids"]
                .iter()
                .filter_map(|key| el.get(*key).and_then(Value::as_array))
                .flatten()
                .find_map(Value::as_str)?;
            Some((id.to_string(), member.to_string()))
        })
        .collect::<BTreeMap<_, _>>()
        .into_iter()
        .filter_map(|(id, member)| Some((id, conn.net_of(&member)?)))
        .collect();

    let mut index = SpatialRoutingIndex::new();
    let mut nets: NetTerminals = BTreeMap::new();

    let board = elements
        .iter()
        .find(|el| type_of(el) == "pcb_board")
        .ok_or("Circuit JSON has no pcb_board to bound routing")?;
    let board: PcbBoardElm =
        serde_json::from_value(board.clone()).map_err(|e| format!("malformed pcb_board: {e}"))?;
    if board.outline.is_none() && (board.width.is_none() || board.height.is_none()) {
        return Err("pcb_board has neither an outline nor width and height".to_string());
    }
    index.set_boundary(board.outline_points().iter().map(|p| [p.x, p.y]).collect());

    for (i, el) in elements.iter().enumerate() {
        let typ = type_of(el);
        let fail = |what: &str| format!("{typ} at JSON index {i}: {what}");
        let x = num(el, "x");
        let y = num(el, "y");
        let net = str_of(el, "pcb_port_id")
            .and_then(|p| port_to_source.get(p))
            .and_then(|sp| conn.net_of(sp));
        match typ {
            "pcb_smtpad" => {
                let (x, y) = x.zip(y).ok_or_else(|| fail("missing x/y"))?;
                let layer = str_of(el, "layer").ok_or_else(|| fail("missing layer"))?;
                let [hw, hh] = pad_half_extent(el).ok_or_else(|| fail("missing pad size"))?;
                index.insert_obstacle(
                    layer,
                    SpatialObstacle::pad(net.clone(), [x - hw, y - hh], [x + hw, y + hh]),
                );
                if let Some(net) = net {
                    nets.entry(net).or_default().push(Terminal {
                        point: [x, y],
                        layers: vec![layer.to_string()],
                    });
                }
            }
            "pcb_plated_hole" => {
                let (x, y) = x.zip(y).ok_or_else(|| fail("missing x/y"))?;
                let [hw, hh] = pad_half_extent(el).ok_or_else(|| fail("missing pad size"))?;
                for layer in &config.layers {
                    index.insert_obstacle(
                        layer,
                        SpatialObstacle::pad(net.clone(), [x - hw, y - hh], [x + hw, y + hh]),
                    );
                }
                if let Some(net) = net {
                    nets.entry(net).or_default().push(Terminal {
                        point: [x, y],
                        layers: config.layers.clone(),
                    });
                }
            }
            "pcb_hole" | "pcb_via" => {
                let (x, y) = x.zip(y).ok_or_else(|| fail("missing x/y"))?;
                let d = [
                    "outer_diameter",
                    "hole_diameter",
                    "hole_width",
                    "hole_height",
                ]
                .iter()
                .filter_map(|k| num(el, k))
                .fold(None, |m: Option<f64>, v| Some(m.map_or(v, |m| m.max(v))))
                .ok_or_else(|| fail("missing diameter"))?;
                for layer in &config.layers {
                    index.insert_obstacle(layer, SpatialObstacle::via(None, [x, y], d));
                }
            }
            "pcb_trace" => {
                let net = str_of(el, "source_trace_id")
                    .and_then(|t| trace_net.get(t).cloned())
                    .or_else(|| str_of(el, "net_id").map(str::to_owned));
                insert_existing_trace(&mut index, el, net, config).map_err(|e| fail(&e))?;
            }
            _ => {}
        }
    }

    Ok((index, nets))
}

fn insert_existing_trace(
    index: &mut SpatialRoutingIndex,
    el: &Value,
    net: Option<String>,
    config: &AStarConfig,
) -> Result<(), String> {
    let route = el
        .get("route")
        .and_then(Value::as_array)
        .ok_or("missing route")?;
    let mut prev: Option<(&str, [f64; 2], f64)> = None;
    for pt in route {
        let p = num(pt, "x")
            .zip(num(pt, "y"))
            .map(|(x, y)| [x, y])
            .ok_or("route point missing x/y")?;
        if str_of(pt, "route_type") == Some("via") {
            let d = num(pt, "outer_diameter").unwrap_or(config.via_diameter);
            for layer in &config.layers {
                index.insert_obstacle(layer, SpatialObstacle::via(net.clone(), p, d));
            }
            prev = None;
            continue;
        }
        let layer = str_of(pt, "layer").ok_or("wire point missing layer")?;
        let width = num(pt, "width").ok_or("wire point missing width")?;
        if let Some((pl, pp, pw)) = prev.filter(|(pl, _, _)| *pl == layer) {
            index.insert_obstacle(pl, SpatialObstacle::trace(net.clone(), pp, p, pw));
        } else {
            index.insert_obstacle(layer, SpatialObstacle::via(net.clone(), p, width));
        }
        prev = Some((layer, p, width));
    }
    Ok(())
}

/// Half width and half height of a pad's axis-aligned bounding box, accounting for `ccw_rotation`.
fn pad_half_extent(el: &Value) -> Option<[f64; 2]> {
    let (w, h) = match (num(el, "width"), num(el, "height")) {
        (Some(w), Some(h)) => (w, h),
        _ => {
            let d = num(el, "outer_diameter")
                .or_else(|| num(el, "radius").map(|r| r * 2.0))
                .map(|d| (d, d));
            let outer = num(el, "outer_width").zip(num(el, "outer_height"));
            let rect = num(el, "rect_pad_width").zip(num(el, "rect_pad_height"));
            d.or(outer).or(rect)?
        }
    };
    let rot = ["ccw_rotation", "rect_ccw_rotation"]
        .iter()
        .find_map(|k| num(el, k))
        .unwrap_or(0.0)
        .to_radians();
    let (s, c) = (rot.sin().abs(), rot.cos().abs());
    Some([(w * c + h * s) / 2.0, (w * s + h * c) / 2.0])
}

fn type_of(el: &Value) -> &str {
    str_of(el, "type").unwrap_or("")
}

fn str_of<'a>(el: &'a Value, key: &str) -> Option<&'a str> {
    el.get(key).and_then(Value::as_str)
}

fn num(el: &Value, key: &str) -> Option<f64> {
    el.get(key)
        .and_then(Value::as_f64)
        .filter(|v| v.is_finite())
}
