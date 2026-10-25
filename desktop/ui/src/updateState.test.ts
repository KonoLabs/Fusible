import { describe, expect, test } from "bun:test";
import { IDLE, reduce, type UpdateEvent, type UpdateState } from "./updateState";
import { checkedLabel, percentOf, releaseUrl, statusText } from "./updateText";

const AVAILABLE: UpdateEvent = {
  type: "checked",
  result: { status: "available", version: "0.19.0", notes: "", date: null },
};

function run(events: UpdateEvent[], from: UpdateState = IDLE): UpdateState {
  return events.reduce(reduce, from);
}

describe("update state", () => {
  test("a manual check goes through checking to the result", () => {
    expect(run([{ type: "check-started" }]).kind).toBe("checking");
    const current = { status: "up_to_date" } as const;
    expect(run([{ type: "check-started" }, { type: "checked", result: current }]).kind).toBe("uptodate");
    expect(run([{ type: "check-started" }, AVAILABLE])).toEqual({ kind: "available", version: "0.19.0" });
  });
  test("the unsupported answer carries the release page", () => {
    const result = { status: "unsupported", url: "https://example.com/r" } as const;
    expect(run([{ type: "checked", result }])).toEqual({ kind: "unsupported", url: "https://example.com/r" });
  });
  test("an automatic check stays silent when skipped or failed", () => {
    expect(run([{ type: "checked", result: { status: "skipped" } }])).toBe(IDLE);
    expect(run([{ type: "check-failed", message: "offline", auto: true }])).toBe(IDLE);
  });
  test("a manual failure keeps the message and offers a retry", () => {
    const state = run([{ type: "check-started" }, { type: "check-failed", message: "offline", auto: false }]);
    expect(state).toEqual({ kind: "error", message: "offline", retry: "check", version: null });
  });
  test("the download reports percent and ends ready", () => {
    const state = run([AVAILABLE, { type: "install-started" }, { type: "progress", downloaded: 50, total: 200 }]);
    expect(state).toEqual({ kind: "downloading", version: "0.19.0", percent: 25 });
    expect(run([{ type: "installed" }], state)).toEqual({ kind: "ready", version: "0.19.0", later: false });
  });
  test("a progress tick that keeps the percent returns the same state", () => {
    const state = run([AVAILABLE, { type: "install-started" }, { type: "progress", downloaded: 50, total: 200 }]);
    expect(reduce(state, { type: "progress", downloaded: 51, total: 200 })).toBe(state);
  });
  test("a failed install keeps the version so the retry can run", () => {
    const failed = run([AVAILABLE, { type: "install-started" }, { type: "failed", retry: "install", message: "bad" }]);
    expect(failed).toEqual({ kind: "error", message: "bad", retry: "install", version: "0.19.0" });
    expect(run([{ type: "install-started" }], failed).kind).toBe("downloading");
  });
  test("a ready update survives a new check and can be deferred", () => {
    const ready = run([AVAILABLE, { type: "install-started" }, { type: "installed" }]);
    expect(run([{ type: "check-started" }, AVAILABLE], ready)).toBe(ready);
    expect(run([{ type: "later" }], ready)).toEqual({ kind: "ready", version: "0.19.0", later: true });
  });
  test("install and progress do nothing without an available update", () => {
    expect(run([{ type: "install-started" }])).toBe(IDLE);
    expect(run([{ type: "progress", downloaded: 1, total: 2 }])).toBe(IDLE);
    expect(run([{ type: "installed" }])).toBe(IDLE);
  });
});

describe("update text", () => {
  test("percentOf handles unknown and oversized totals", () => {
    expect(percentOf(10, null)).toBeNull();
    expect(percentOf(10, 0)).toBeNull();
    expect(percentOf(300, 200)).toBe(100);
  });
  test("statusText names the version and progress", () => {
    expect(statusText({ kind: "available", version: "0.19.0" })).toBe("Version 0.19.0 is available");
    expect(statusText({ kind: "downloading", version: "0.19.0", percent: 40 })).toBe("Downloading version 0.19.0, 40%");
    expect(statusText(IDLE)).toBe("");
  });
  test("checkedLabel and releaseUrl", () => {
    const now = Date.parse("2026-10-24T12:00:00Z");
    expect(checkedLabel(null, now)).toBe("Never checked");
    expect(checkedLabel(now / 1000 - 30, now)).toBe("Checked just now");
    expect(checkedLabel(now / 1000 - 3 * 3600, now)).toBe("Checked 3 hours ago");
    expect(checkedLabel(now / 1000 - 5 * 86400, now)).toBe("Checked 5 days ago");
    expect(releaseUrl("0.19.0")).toBe("https://github.com/KonoLabs/Fusible/releases/tag/v0.19.0");
  });
});
