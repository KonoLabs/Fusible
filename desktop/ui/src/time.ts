function toMillis(value: string): number {
  if (/^\d+$/.test(value)) {
    const n = Number(value);
    return n < 1e12 ? n * 1000 : n;
  }
  return Date.parse(value);
}

const UNITS: [string, number][] = [
  ["year", 31536000],
  ["month", 2592000],
  ["week", 604800],
  ["day", 86400],
  ["hour", 3600],
  ["minute", 60],
];

export function relativeTime(value: string | null, now: number = Date.now()): string {
  if (!value) return "Never opened";
  const then = toMillis(value);
  if (Number.isNaN(then)) return "Never opened";
  const seconds = Math.max(0, Math.floor((now - then) / 1000));
  if (seconds < 60) return "Opened just now";
  for (const [unit, size] of UNITS) {
    const count = Math.floor(seconds / size);
    if (count >= 1) return `Opened ${count} ${unit}${count > 1 ? "s" : ""} ago`;
  }
  return "Opened just now";
}
