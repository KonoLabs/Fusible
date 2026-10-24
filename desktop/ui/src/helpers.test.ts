import { describe, expect, test } from "bun:test";
import { countsLabel } from "./labels";
import { finalPath, shortenPath, suggestName } from "./path";
import { toolchainReport } from "./report";
import { relativeTime } from "./time";
import { validateName, validatePort } from "./validate";

describe("validateName", () => {
  test("accepts ordinary names", () => {
    expect(validateName("FieldCam v2")).toBeNull();
    expect(validateName("  motor-driver ")).toBeNull();
  });
  test("rejects empty, unsafe and reserved names", () => {
    expect(validateName("   ")).not.toBeNull();
    expect(validateName("a/b")).not.toBeNull();
    expect(validateName("a:b")).not.toBeNull();
    expect(validateName("name.")).not.toBeNull();
    expect(validateName("CON")).not.toBeNull();
    expect(validateName("nul.txt")).not.toBeNull();
    expect(validateName("..")).not.toBeNull();
    expect(validateName("x".repeat(81))).not.toBeNull();
  });
});

describe("finalPath", () => {
  test("appends the name when creating a folder", () => {
    expect(finalPath("/home/a/Fusible/", "Board", true)).toBe("/home/a/Fusible/Board");
    expect(finalPath("C:\\Users\\a\\Fusible", "Board", true)).toBe("C:\\Users\\a\\Fusible\\Board");
  });
  test("keeps the location when not creating a folder", () => {
    expect(finalPath("/home/a/Existing", "Board", false)).toBe("/home/a/Existing");
  });
  test("falls back to the location while the name is empty", () => {
    expect(finalPath("/home/a", " ", true)).toBe("/home/a");
  });
});

describe("relativeTime", () => {
  const now = Date.parse("2026-10-24T12:00:00Z");
  test("handles missing and recent values", () => {
    expect(relativeTime(null, now)).toBe("Never opened");
    expect(relativeTime("2026-10-24T11:59:40Z", now)).toBe("Opened just now");
  });
  test("picks the largest unit", () => {
    expect(relativeTime("2026-10-24T11:55:00Z", now)).toBe("Opened 5 minutes ago");
    expect(relativeTime("2026-10-24T11:00:00Z", now)).toBe("Opened 1 hour ago");
    expect(relativeTime("2026-10-21T12:00:00Z", now)).toBe("Opened 3 days ago");
    expect(relativeTime("2026-08-24T12:00:00Z", now)).toBe("Opened 2 months ago");
  });
  test("accepts epoch seconds", () => {
    expect(relativeTime(String(Math.floor(now / 1000) - 7200), now)).toBe("Opened 2 hours ago");
  });
});

describe("toolchainReport", () => {
  test("lists found and missing tools", () => {
    const text = toolchainReport({ bun: { path: "/b/bun", version: "1.2.0" }, node: null, tsci: null }, "0.1.0");
    expect(text).toContain("Fusible Studio 0.1.0");
    expect(text).toContain("bun: 1.2.0 (/b/bun)");
    expect(text).toContain("node: not found");
    expect(text).toContain("bundled bun: none");
  });
});

describe("display helpers", () => {
  test("shortenPath keeps short paths and elides long ones", () => {
    expect(shortenPath("/home/a/Fusible")).toBe("/home/a/Fusible");
    const long = shortenPath("/home/yann/Code/Kono/Fusible/projects/FieldCam", 36);
    expect(long.length).toBeLessThanOrEqual(36);
    expect(long.endsWith("projects/FieldCam")).toBe(true);
  });
  test("countsLabel pluralises and skips zeros", () => {
    expect(countsLabel({ boards: 2, firmware: 1, bodies: 1 })).toBe("2 boards, 1 firmware, 1 body");
    expect(countsLabel({ boards: 1, firmware: 0, bodies: 3 })).toBe("1 board, 3 bodies");
    expect(countsLabel({ boards: 0, firmware: 0, bodies: 0 })).toBe("No boards yet");
  });
  test("suggestName and validatePort", () => {
    expect(suggestName("/home/a/My Board")).toBe("My Board");
    expect(validatePort(3020)).toBeNull();
    expect(validatePort(80)).not.toBeNull();
    expect(validatePort(70000)).not.toBeNull();
  });
});
