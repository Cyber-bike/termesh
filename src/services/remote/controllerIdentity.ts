export function normalizeControllerIdentitySeed(value: unknown): number[] | null {
  if (!Array.isArray(value) || value.length !== 32) return null;
  const bytes: unknown[] = value;
  if (!bytes.every((byte): byte is number =>
    typeof byte === 'number' && Number.isInteger(byte) && byte >= 0 && byte <= 255)) {
    return null;
  }
  return [...bytes];
}