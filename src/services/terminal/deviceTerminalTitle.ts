export function buildDeviceTerminalTitle(
  deviceName: string,
  noteName: string | null,
  fallbackTitle: string,
): string {
  const normalizedDeviceName = deviceName.trim() || fallbackTitle;
  const normalizedNoteName = noteName?.trim() || fallbackTitle;
  return `${normalizedDeviceName} · ${normalizedNoteName}`;
}