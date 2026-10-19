# Changelog

All notable changes to this project are documented here. The format is
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html). While on
`0.x`, a breaking change bumps the minor.

## [Unreleased]

## [0.1.0] - 2026-10-19

### Added

- CLI `fusible` with `--input` (Circuit JSON path) and `--mcu` (silicon target,
  accepted and echoed in phase 1, consumed by pin-muxing in phase 2).
- Partial Circuit JSON parser on a Serde internally-tagged enum
  (`#[serde(tag = "type")]`): components, nets, ports, pads and traces are
  typed; unknown element types are skipped instead of failing the parse.
- Rules engine: `Rule` trait, `Linter` registry, and the first rule
  `unconnected_net` (a net routed to fewer than two pads/ports blows the fuse).
- miette diagnostics end to end: parse errors point at the offending line in
  the source file, violations render as related errors under
  `le fusible a sauté`, and any failure exits non-zero so codegen never runs.
- CI workflow (fmt, clippy `-D warnings`, test, release build) and a release
  workflow producing darwin amd64/arm64 + linux amd64 tarballs with checksums.
