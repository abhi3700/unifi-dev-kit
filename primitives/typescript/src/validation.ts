export function isUniFiUserId(value: string): boolean {
  return value.length === 24 && /^[0-9a-fA-F]+$/.test(value);
}
