import { FORBIDDEN } from "./path";

const RESERVED = /^(con|prn|aux|nul|com[1-9]|lpt[1-9])(\..*)?$/i;
const MAX_NAME = 80;

export function validateName(raw: string): string | null {
  const name = raw.trim();
  if (!name) return "Enter a name.";
  if (name.length > MAX_NAME) return `Use at most ${MAX_NAME} characters.`;
  if (FORBIDDEN.test(name)) return 'A name cannot contain < > : " / \\ | ? *';
  if (/[. ]$/.test(name)) return "A name cannot end with a dot or a space.";
  if (/^\.+$/.test(name)) return "A name cannot be only dots.";
  if (RESERVED.test(name)) return "This name is reserved by Windows.";
  return null;
}

export function validatePort(value: number): string | null {
  if (!Number.isInteger(value) || value < 1024 || value > 65535) return "Use a port between 1024 and 65535.";
  return null;
}
