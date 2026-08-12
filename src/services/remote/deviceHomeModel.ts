import type { DeviceConnectionStatus } from './deviceConnections.ts';
import type { PairedDevice } from './pairedDeviceStore.ts';

export type DeviceHomeCard =
  | { kind: 'add' }
  | { kind: 'local' }
  | { kind: 'remote'; device: PairedDevice; status: DeviceConnectionStatus };

export interface DeviceStatusReader {
  status(nodeId: string): DeviceConnectionStatus;
}

export function buildDeviceHomeCards(
  devices: PairedDevice[],
  statuses: DeviceStatusReader,
): DeviceHomeCard[] {
  return [
    { kind: 'add' },
    { kind: 'local' },
    ...devices.map((device): DeviceHomeCard => ({
      kind: 'remote',
      device,
      status: statuses.status(device.nodeId),
    })),
  ];
}

export function getRefreshNodeIds(cards: DeviceHomeCard[]): string[] {
  return cards.flatMap((card) => {
    if (card.kind !== 'remote' || card.status.state === 'connected' || card.status.state === 'connecting') {
      return [];
    }
    return [card.device.nodeId];
  });
}