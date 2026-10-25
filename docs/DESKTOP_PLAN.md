# Plan: Fusible Studio desktop app

Goal: an electrical engineer on Windows installs Fusible Studio by double-clicking a setup file. She never opens a terminal. The app opens on a projects page, then shows the studio she already knows.

Verified against the tree at `1a8128e` (v0.14.3) and Vitrine at `0df4dc5` on 2026-10-24. Paths and line numbers below were read, not recalled.

## Summary

- A Tauri 2 shell in `desktop/` runs the existing `fusible studio` binary as a sidecar, one per open project, and points its window at it. The studio itself does not move or change shape.
- The shell owns two local pages: Projects (cards grid, new, open, recent) and Settings.
- The installer also ships `bun`, so TSX compiles and `bun install` work on a bare Windows machine.
- `bom.yml` already stores `currency` per project. One gap remains: price estimates are euros whatever the currency says.
- Vitrine gets `/fusible` (landing page) and `/fusible/download` (installers). No new website.

## Facts the design rests on

| Fact | Where | Consequence |
|---|---|---|
| `fusible` is a binary crate only, no `lib.rs` | `src/main.rs:8-23` | The shell cannot call the dev server as a library without a refactor. |
| `POST /api/run` re-runs `std::env::current_exe()` | `src/dev/run_api.rs:46` | Linking the server into the Tauri binary would make RUNS launch the desktop app. A sidecar avoids this. |
| The manifest is found from the working directory | `FusibleConfig::nearest()`, `src/dev/run_api.rs:42`; `Runner::detect`, `src/tscircuit/mod.rs:65` | Start the sidecar with `cwd` set to the project folder. |
| The server binds 127.0.0.1, tries 20 ports upward, prints the URL | `src/dev/bind.rs:14`, `src/dev/server.rs:136` | The shell needs a stable, machine-readable ready line instead of parsing the emoji line. |
| Host must be loopback; `/api/*` and `/events` need same-origin | `src/dev/server.rs:52-63`, `src/dev/guard.rs` | Navigating the window to `http://localhost:<port>/` passes the guards unchanged. An iframe inside the Tauri page would send `Sec-Fetch-Site: cross-site` on navigation and is not worth the fight. |
| Runner detection finds only `tsci` in `node_modules/.bin` or on PATH | `src/tscircuit/detect.rs:16`, `RunnerKind` in `src/tscircuit/mod.rs:29` | A bare Windows machine has neither. Needs a bundled toolchain. |
| `fusible init` installs deps with `bun` then `npm` from PATH | `src/commands/init.rs:126` | Must respect the bundled bun. |
| Body generator already shells out to bun | `src/commands/body/generator_setup.rs:42` | Same: respect the bundled bun. |
| Release builds macOS x2 and Linux, no Windows | `.github/workflows/release.yml:17-32` | Windows is not built or tested today. This is the real first step. |
| `bom.yml` has `currency`, seeded to `EUR` on first read | `src/bom/file.rs:73`, `src/bom/file.rs:126`, studio picker `src/assets/bom.js:81` | Per-project currency exists since v0.14.3. |
| Estimates are euros | `src/bom/estimate.rs:10` | A USD project shows euro estimates labeled USD. Bug. |
| Vitrine is SvelteKit 2, Svelte 5, Tailwind 4, on Vercel; has `/fieldcam` and a home `Fusible.svelte` section | `../Vitrine/src/routes/fieldcam/+page.svelte`, `../Vitrine/src/lib/components/sections/Fusible.svelte` | `/fusible` follows the `/fieldcam` pattern. |
| `KonoLabs/Fusible` is public | `gh repo view` | `releases/latest/download/<asset>` links work without an API call. |

## Decisions for you

Each has a recommendation. Agents should build the recommended option unless you say otherwise.

1. **Toolchain on Windows.** Recommended: bundle `bun` as a second sidecar (~45 MB compressed in the installer) and add a `Bun` runner kind. Alternative: a "Install toolchain" button that downloads bun on first launch. Bundling works offline and on locked-down machines; downloading keeps the installer small.
2. **Code signing.** Unsigned Windows installers show a SmartScreen warning ("More info", "Run anyway"). Unsigned macOS apps need right-click "Open" and are blocked outright when downloaded through some browsers. Recommended for now: ship unsigned, document the one-click workaround on the download page, add signing later. macOS notarization needs an Apple Developer account (99 USD a year); Windows signing needs a certificate (Azure Trusted Signing is the cheapest route).
3. **Auto-update.** Recommended: later phase. The Tauri updater needs a signing keypair and a `latest.json` on each release. Phase 4 below.
4. **Website.** Recommended: add pages to Vitrine in its current design language, no redesign. A separate site doubles the deploy surface for two pages.
5. **Estimate currency.** Recommended: a fixed EUR rate table in `src/bom/estimate.rs` for the 12 currencies the studio offers, applied to estimates only. User prices in `bom.yml` are never converted. Estimates are already rough; a yearly-updated table is honest enough and stays offline.

## Architecture

```text
Fusible Studio.exe (Tauri 2, desktop/src-tauri)
├── window "main"
│   ├── tauri://localhost/            Projects page   (desktop/ui)
│   ├── tauri://localhost/#settings   Settings page   (desktop/ui)
│   └── http://localhost:<port>/      the studio, served by the sidecar
├── sidecar: fusible studio --desktop     cwd = project folder, one at a time
└── sidecar: bun                       found by fusible through FUSIBLE_BUN
```

Opening a project:

1. The shell kills the current sidecar, if any.
2. It spawns `fusible studio --desktop -p <settings.port>` with `cwd` = project folder and env `FUSIBLE_BUN=<bundled bun path>`.
3. It reads stdout until the line `FUSIBLE_READY {"url":"http://localhost:3020/","port":3020}`.
4. It navigates the window to that URL, and pushes the project to the recent list.
5. On any exit before ready, it shows the captured stderr on the Projects page with a "Copy log" button.

Going back: a native menu item "File > Projects" (Ctrl+Shift+O / Cmd+Shift+O) and a "Projects" button in the studio. The button exists only when the studio runs inside the shell. It calls one Tauri command, `open_projects`, which the capability file allows from `http://localhost:*` and nothing else.

Quitting: the shell kills the sidecar on `RunEvent::Exit`. Because a crashed shell cannot kill anything, `fusible studio --desktop` also exits when its stdin closes. The shell holds the sidecar's stdin open and never writes to it.

One window, one project at a time. Multi-window is out of scope.

## Contracts between workstreams

Agents build against these, so they can work in parallel. Change them only in this file.

### Sidecar CLI (workstream B builds, C consumes)

- `fusible studio --desktop` does three things, nothing else:
  - prints exactly one line `FUSIBLE_READY {json}` to stdout once the server is bound, before the first compile finishes. JSON keys: `url`, `port`.
  - exits with code 0 when stdin reaches EOF.
  - sets `window.FUSIBLE_DESKTOP = true` in the served `viewer.html`, so the studio shows its Projects button.
- `fusible init --name <NAME> [--currency <CODE>]` in an empty or existing folder: unchanged behavior, plus `--currency` writes `bom.yml` with that currency. Exit 0 on success, 2 on error, stderr readable as one message.
- `fusible doctor --json` prints `{"bun": {"path", "version"} | null, "node": ... | null, "tsci": ... | null}` and exits 0 even when tools are missing.
- Env `FUSIBLE_BUN=<path>`: when set and the file exists, every place that runs bun uses it, and runner detection falls back to a `Bun` runner (`<bun> x --bun tsci ...`) when no `tsci` is found.

### Tauri commands (workstream C builds, D consumes)

All return `Result<T, String>`. The error string is shown to the user as is, so write it for a human.

| Command | Args | Returns |
|---|---|---|
| `list_projects` | none | `Project[]`: recent plus folders with `fusible.yml` one level under `settings.projects_dir`, recent first, deduplicated by canonical path |
| `open_project` | `path` | `()`, navigates the window on success |
| `create_project` | `parent`, `name`, `create_folder: bool`, `currency` | `Project` |
| `inspect_folder` | `path` | `{ has_manifest: bool, name: string \| null }` |
| `forget_project` | `path` | `()` removes it from recent only, never touches disk |
| `reveal_project` | `path` | `()` opens the OS file manager |
| `get_settings` / `set_settings` | `Settings` | `Settings` |
| `toolchain_status` | none | output of `fusible doctor --json` plus the bundled bun version |
| `open_projects` | none | `()` stops the sidecar, navigates to the Projects page |
| `last_error` | none | `string \| null`, the stderr of the last failed open |

```ts
type Project = {
  path: string; name: string;
  boards: number; firmware: number; bodies: number;
  last_opened: string | null; missing: boolean;
};
type Settings = {
  projects_dir: string;
  default_currency: string;
  port: number;
  open_last_project: boolean;
  theme: "system" | "dark" | "light";
};
```

`name`, `boards`, `firmware`, `bodies` come from parsing `fusible.yml` with a minimal serde struct in the shell (top-level `name`, and the lengths of the `boards`, `firmware`, `bodies` maps). Unknown keys are ignored. An unreadable manifest gives a card named after the folder with zero counts. `missing` is true when the folder no longer exists; the card shows "Folder not found" with a Forget button.

Storage: `tauri-plugin-store`, files `settings.json` and `recent.json` in the app data dir. Recent keeps 20 entries.

### Release asset names (workstream F builds, G consumes)

Stable names, no version, so `https://github.com/KonoLabs/Fusible/releases/latest/download/<name>` always works:

| OS | Asset |
|---|---|
| Windows x64 | `Fusible-Studio-windows-x64-setup.exe` (NSIS, per-user, no admin) |
| macOS (Apple Silicon and Intel) | `Fusible-Studio-macos-universal.dmg` |
| Linux x64 | `Fusible-Studio-linux-x64.AppImage`, `Fusible-Studio-linux-x64.deb` |
| CLI Windows | `fusible_<version>_windows_amd64.zip` (matches the existing CLI naming) |

## Workstreams

Dependency order: A first. B, C, D, G in parallel once A is merged (C and D only need the contracts). E is folded into B. F after A, B, C.

```text
A ──┬── B ──┐
    ├── C ──┼── F
    ├── D ──┘
    └── G
```

### A. Windows builds of the CLI

Files: `.github/workflows/ci.yml`, `.github/workflows/release.yml`, test gating in `src/**/*_tests.rs`, any `cfg(windows)` fixes it uncovers.

1. Add a `windows-latest` job to `ci.yml` running fmt, clippy, `cargo test --locked`. Expect failures; the point is to see them.
2. Known suspects: tests that spawn `sh` (`src/dev/runs_tests.rs:8`, `src/dev/run_exec_tests.rs:8`, `src/commands/body/generator_tests.rs:7`) need `#[cfg(unix)]` or a `cmd /C` twin. Paths compared as strings with `/`. `src/commands/build.rs:83` already branches on `cmd`.
3. Add `x86_64-pc-windows-msvc` to the release matrix, packaged as `.zip` with `fusible.exe`.
4. Fix what breaks. Do not paper over a failing test with `cfg(unix)` when the behavior matters on Windows (path handling, process spawning, file watching).

Done when: CI is green on Windows and a manual `fusible studio examples/<board>.json` on a Windows VM serves the studio.

### B. Desktop hooks in the CLI and the bom currency fix

Files: `src/cli/args.rs`, `src/commands/dev.rs`, `src/dev/server.rs`, `src/dev/assets.rs`, `src/assets/viewer.html`, `src/tscircuit/{mod,detect,command}.rs`, `src/commands/init.rs`, `src/commands/doctor.rs`, `src/commands/body/generator*.rs`, `src/bom/{estimate,report,file}.rs`, `CHANGELOG.md`.

1. `--desktop` flag on `dev`: the ready line, the stdin-EOF watchdog thread, and the `FUSIBLE_DESKTOP` flag injected when serving `viewer.html`.
2. Studio Projects button: shown only when `window.FUSIBLE_DESKTOP`. It calls `window.__TAURI_INTERNALS__.invoke('open_projects')` so the studio needs no Tauri npm package. Put it in the existing dock, same style as the other dock buttons.
3. One helper `bun_path()` that reads `FUSIBLE_BUN`, then PATH. Use it in `init.rs:126`, the body generator, and doctor. Grep every `"bun"` literal; fix the shared lookup once.
4. `RunnerKind::Bun(PathBuf)`: `base_command` becomes `<bun> x --bun tsci`. Detection order stays local, global, then Bun. Verify on Windows that `bun x --bun tsci build` compiles a board without node installed; if it does not, stop and report, because decision 1 depends on it.
5. `fusible init --currency <CODE>` writes `bom.yml` through the existing `BomFile` type, so the format stays in one place. Validate the code against the same list the studio offers.
6. `fusible doctor --json` as specified.
7. Estimate conversion (decision 5): estimates convert from EUR with a const table; user prices and `print.price_per_kg` never convert. Unknown currency keeps euros and the report says so. One test: a USD file yields an estimate equal to the EUR estimate times the table rate.
8. Regression tests per item. Studio button: extend `src/dev/studio_tests.rs` style checks that the button is absent without the flag.

Done when: the four gates pass, and `fusible studio --desktop` in FieldCam prints the ready line, serves the button, and exits when its stdin closes.

### C. Tauri shell

Files: everything under `desktop/src-tauri/`. Nothing outside `desktop/`.

1. `desktop/src-tauri` is its own crate with its own `Cargo.lock`. Not a workspace member, so the `fusible` crate, its tests, and its crates.io package (`Cargo.toml` `include` list) stay untouched.
2. Tauri 2 with plugins `shell` (sidecars), `dialog`, `store`, `opener`, `single-instance`. Nothing else.
3. `externalBin`: `binaries/fusible` and `binaries/bun`, with the per-triple names Tauri expects. A `desktop/scripts/fetch-sidecars.ts` (bun) copies `target/release/fusible` and downloads the pinned bun release for the host triple. Pin the bun version in one place.
4. Implement every command in the contract table. Sidecar lifecycle as in Architecture, behind one `Studio` struct held in Tauri state. Opening a second project while one runs replaces it.
5. Capabilities: `default.json` for the local pages with only the plugins above; `studio.json` with `remote.urls: ["http://localhost:*"]` granting `open_projects` only.
6. Native menu: File (Projects, Open folder, New project, Settings, Quit), View (Reload, Zoom in/out/reset), Help (Fusible docs, Report a bug, About). Devtools in debug builds only.
7. Window: 1440x900 default, min 960x600, remembers size via the store. Title is "Fusible Studio" on Projects, "<project name> - Fusible Studio" in the studio.
8. Windows bundle: NSIS, `installMode: currentUser`, WebView2 `embedBootstrapper` so a Windows 10 box without WebView2 still installs.
9. Rust here follows the repo rules: no inline comments, one doc block per public fn, filet clean.

Done when: `bun tauri dev` in `desktop/` opens Projects, can create a project, open it, see the studio compile, go back, and quit with no `fusible` process left behind (check with Task Manager or `ps`).

### D. Projects and Settings pages

Files: everything under `desktop/ui/`.

1. Plain HTML, CSS, and TypeScript, bundled with `bun build`, same as the studio's own assets. No framework: three views do not need one.
2. Reuse the studio look: copy the color tokens and the Undefined font from `src/assets/viewer.html` and `src/assets/undefined-medium.otf`. Never use a bold weight of Undefined. Icons from pixelarticons, as in the studio.
3. **Projects page**
   - Header: "Projects", buttons "New project" (+), "Open folder", a gear for Settings.
   - Recent strip: the five most recent as compact rows above the grid.
   - Cards grid: one card per `Project`. Shows name, shortened path, counts ("2 boards, 1 firmware, 1 body"), "Opened 3 days ago". Click opens. A kebab menu has Reveal in folder and Forget.
   - Empty state: one sentence and the two buttons. This is what she sees on first launch, so make it the best screen in the app.
   - Opening state: the card shows a spinner until navigation. On failure, an error panel with the message from `last_error` and a "Copy log" button.
4. **New project dialog**: name (required, folder-safe), location (defaults to `projects_dir`, Browse button), "Create a folder named <name>" checkbox (on by default), currency (defaults to `default_currency`). Shows the final path live. Create calls `create_project`, then `open_project`.
5. **Open folder**: native dialog. If `inspect_folder` says there is no manifest, ask "This folder is not a Fusible project yet. Create one here?" and route to the dialog with location prefilled and the folder checkbox off.
6. **Settings page**: projects folder, default currency for new projects, studio port, open last project on launch, theme. A Toolchain section shows `toolchain_status` read-only with a Copy button for bug reports. A footer shows app and fusible versions.
7. Keyboard: Enter opens the focused card, Ctrl/Cmd+N new project, Ctrl/Cmd+O open folder, Esc closes dialogs. Every control reachable by Tab with a visible focus ring.
8. Develop against a mock `invoke` that returns fixture data, so this work does not wait on C.

Done when: every state above (empty, many projects, missing folder, opening, failure) renders correctly at 960x600 and 1920x1080, light and dark.

### F. Desktop release pipeline

Files: `.github/workflows/desktop.yml`, `desktop/src-tauri/tauri.conf.json` bundle section.

1. Triggered by the same `v*` tag as `release.yml`. Matrix: `windows-latest`, `macos-latest` (universal: build both Apple triples, `lipo` the fusible sidecar), `ubuntu-22.04` (older glibc so the AppImage runs on more distros).
2. Steps: build the studio bundle, build `fusible` release, fetch bun, `tauri build`, rename outputs to the stable asset names, `gh release upload --clobber` onto the tag's release. The CLI `release.yml` creates the release; this job waits for it or creates it if missing, with the same fallback pattern as `release.yml`.
3. The desktop version equals the crate version. Bump both in the release commit; add a check that fails the job when `desktop/src-tauri/tauri.conf.json` and `Cargo.toml` disagree.
4. Smoke step per OS: run the bundled `fusible --version` sidecar from the built bundle.

Done when: a test tag on a fork produces all four installers on one release, and the Windows installer installs and launches on a clean Windows 11 VM without admin rights.

### G. Website: `/fusible` and `/fusible/download` in Vitrine

Files: under `../Vitrine/`: `src/routes/fusible/+page.svelte`, `src/routes/fusible/download/+page.svelte`, `src/lib/components/sections/Fusible.svelte`, `src/lib/components/sections/TopNav.svelte`, `static/fusible/`.

1. **Landing `/fusible`**, same structure and tone as `/fieldcam`, with `Seo`:
   - Hero: "The fuse for your hardware." One line on what it does, a Download button that links to `/fusible/download`, a secondary "Install the CLI" anchor.
   - What it checks: the six rules from the fuse contract (unconnected net, ghost net, shorted nets, malformed element, trace clearance, pin mux), one line each.
   - Studio: the four screenshots already in `static/fusible/` (PCB, diagnostics, schematic, 3D), plus parts and costs once there is a screenshot.
   - One manifest: the `fusible.yml` story (boards, firmware, bodies).
   - Exports: fab, cad, docs presets.
   - CLI: `cargo install fusible` and the quickstart from the home section.
   - Link to GitHub and FieldCam as the reference board.
2. **Download `/fusible/download`**:
   - Detects the visitor's OS from `navigator.userAgent` on the client and puts that installer first. All others stay listed below.
   - Links use `https://github.com/KonoLabs/Fusible/releases/latest/download/<asset>` from contract F. No API call needed for the links.
   - Shows the version from `https://api.github.com/repos/KonoLabs/Fusible/releases/latest` fetched in a `+page.server.ts` with a one-hour cache header; on failure the page still renders without a version.
   - Per-OS first-run notes for the unsigned builds (decision 2): Windows "More info, Run anyway"; macOS right-click Open; Linux `chmod +x` for the AppImage.
   - A "Release notes" link to the GitHub release.
3. Home section `Fusible.svelte`: add a Download button next to the install snippet, and a "Read more" link to `/fusible`. TopNav: the Fusible entry goes to `/fusible` from other pages, keeps `#fusible` on the home page.
4. Follow Vitrine's existing styles and theme utilities (commit `09cad30` moved everything to theme color utilities). No new dependency.

Done when: `bun run build` passes, both pages work at 375 px and desktop widths, and every download link resolves (they 404 until F ships once; test against a fork's release or check the URL shape).

### H. Sync panel (git for non-developers)

Wave two. Starts after A to D are merged, because it edits the dev server and the studio sidebar.

Files: `src/dev/git*.rs` (new, small files), `src/dev/server.rs` routes, `src/assets/git.js` (new), `src/assets/viewer.html` sidebar entry, `src/commands/git/` if a CLI twin is wanted (not required).

1. The panel appears in the left sidebar only when the project root is inside a git work tree. It is called "Sync" in the UI. The word "commit" appears nowhere; the word "git" only in Settings and error details.
2. Plumbing: run the real `git` binary through a single `run_git(args)` helper with a 30 s timeout and no shell. Never read a credential. `GIT_TERMINAL_PROMPT=0` so a missing credential fails fast with a readable message instead of hanging.
3. API (same-origin guarded like `/api/run`, JSON POST for writes):
   - `GET /api/git`: `{ available, branch, upstream, ahead, behind, changes: [{path, kind}], last_saves: [{id, author, when, message, files}], conflicts: [path] }`.
   - `POST /api/git/save` `{message}`: stage all, commit, `pull --rebase --autostash`, push. Returns the new status or a human error.
   - `POST /api/git/update`: `pull --rebase --autostash`.
   - `POST /api/git/resolve` `{path, keep: "mine" | "theirs"}`: for files in conflict.
4. Studio UI: status line, message box, Save button, Get updates button, last 20 saves, conflict cards that name the file and the other author and offer Keep mine / Keep theirs. After any update that changes `.kicad_pcb`, `.kicad_sch` or `hardware/**`, trigger a rebuild (the file watcher already does this; verify it fires).
5. If `git` is missing (Windows): the panel shows "Install Git to sync your work" with a link to the official download page and a Re-check button. Do not bundle Git in this wave. If the repo has no remote, Save commits only and says "Saved on this computer. Add a remote to share."
6. Desktop: `toolchain_status` also reports the git version, and the Settings Toolchain section shows it.
7. Tests: build throwaway repos in a temp dir (two clones of a bare repo) to cover save, update with remote changes, a same-file conflict with keep mine and keep theirs, no remote, and git missing (PATH emptied).

Done when: two clones of one bare repo can exchange work from two studio instances with no terminal, including a conflict on a `.kicad_pcb`.

### I. KiCad watch and sync

Wave two, parallel with H. Different files from H.

Files: `src/kicad/*`, `src/commands/sync.rs`, `src/dev/kicad_watch*.rs` (new), `fusible.yml` config for the KiCad file location (`src/config/`).

Scope is option 2 plus 3 from the discussion: KiCad owns placement, the TSX owns the netlist. A full KiCad-to-TSX converter is out of scope.

1. Manifest: a board may name its KiCad file, `kicad: hardware/board.kicad_pcb`. Without it, a single `.kicad_pcb` next to the entry is used.
2. Watch it while the studio runs. On a settled change (the existing debouncer), run the sync logic in place: read the footprints, compare with the circuit, write the position updates back into the TSX the way `fusible sync` does today, then let the normal rebuild run.
3. Never lose her work: before writing the TSX, copy the previous file to `.fusible/backup/<name>-<timestamp>.tsx`, keep the last 10.
4. If the KiCad file adds, removes, or renames a part or changes a net, do not guess. Show it in the studio as a "KiCad differs from the design" notice with the list, and do not write anything for those items. Positions are the only auto-synced data.
5. Fix the known gap from `HANDOFF.md`: `compare_kicad_with_circuit` treats unparseable Circuit JSON as empty (`src/commands/diff.rs`, `src/commands/sync.rs`). Fail closed with an error before the watcher relies on it.
6. Studio: a small status chip "KiCad: in sync / 3 differences / error" near the board health pill, and a SYNC event on the existing SSE stream.
7. Tests: a fixture `.kicad_pcb` with moved footprints updates the TSX; a renamed part is reported and not written; backup rotation keeps 10; invalid Circuit JSON is an error.

Done when: with the studio open on FieldCam, moving a footprint in KiCad and saving updates the TSX position, rebuilds the board, and the chip returns to "in sync" within a few seconds.

## Out of scope

- Multiple projects open at once, tabs, or multiple windows.
- Editing TSX inside the app. She edits in her own editor; the studio hot-reloads.
- A code-signing setup, until decision 2 changes.
- Auto-update, until phase 4.
- Bundling Git on Windows, and a KiCad-to-TSX converter (parts, nets).
- Linux ARM and Windows ARM builds.

## Phase 4 (after the first desktop release)

- Tauri updater with a signing key, `latest.json` uploaded by F.
- Code signing on Windows and macOS.
- Card thumbnails: the sidecar already renders `pcb.svg`; cache one per project on close.
- Last fuse state per card, recorded by the shell from the studio's board health on close.

## Rules for every change

- Conventional Commits, no gitmoji, no co-author lines, no inline comments, no comments inside function bodies.
- One regression test per fix, confirmed failing before and passing after.
- Before any commit in this repo, all four must pass and filet must print nothing:
  ```sh
  cargo fmt --all -- --check
  cargo clippy --all-targets -- -D warnings
  cargo test --locked
  filet check .
  ```
- For `desktop/src-tauri`, run the same fmt and clippy from that directory. Add `desktop/ui/dist`, `desktop/src-tauri/target`, `desktop/src-tauri/gen`, and `desktop/src-tauri/binaries` to `.gitignore` and to `filet.yml` ignores.
- Agents work in separate worktrees by file area as listed per workstream, then merge sequentially. `CHANGELOG.md` is the one shared file: add entries under Unreleased, resolve conflicts by keeping both.
- FieldCam (`../FieldCam`) is the acceptance project. Opening it in the desktop app must compile every board and `fusible check` there must still exit 0.
