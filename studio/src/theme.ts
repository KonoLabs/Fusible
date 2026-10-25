/** Reads a CSS custom property of the document root, the single source being `theme.css`. */
export function themeColor(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(`--${name}`).trim();
}

/** Reads a CSS custom property holding a `#rrggbb` colour as the number three.js expects. */
export function themeHex(name: string): number {
  return parseInt(themeColor(name).replace("#", ""), 16);
}
