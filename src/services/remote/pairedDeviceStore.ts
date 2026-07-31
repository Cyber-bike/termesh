/**
 * Locally-persisted "已配对设备" list (v2.0 doc 5.2).
 *
 * v2.0 has no cloud device registry: the control end is the only place that
 * remembers which devices it has ever paired with. Doc 5.2 fixes the record
 * shape as `{ name, ticket, nodeId, lastConnectedAt, lastKnownOnline }`, keyed
 * by `nodeId` so re-adding the same device (e.g. pasting its connection code
 * again after a network change) updates the existing entry instead of
 * duplicating it.
 *
 * This module only owns the list and its life cycle. Turning a pasted
 * connection code into a `nodeId`/ticket pair is the iroh binding's job (doc
 * 14 phase A0/A) and is deliberately not implemented here yet - same scoping
 * split as `agent/src/identity.rs` on the Agent side: build the storage layer
 * against a shape that is already fixed by the doc, and keep it usable and
 * tested independently of the still-unverified networking integration.
 *
 * Not wired to Obsidian's `loadData`/`saveData` here on purpose: this class
 * takes and returns plain JSON-safe objects, so whichever code eventually owns
 * the plugin's `data.json` can persist it without this module needing to know
 * about the Obsidian API (and without needing an Obsidian test harness to
 * exercise it).
 */

export interface PairedDevice {
  /** Device-chosen or user-given label, shown in the device list UI. */
  name: string;
  /** Opaque connection code (iroh `NodeTicket`) as pasted by the user. Kept in
   * full, not just a summary: doc 5.2 requires it for reconnection, since it
   * carries the address hints the discovery-network fallback needs. */
  ticket: string;
  /** Stable identity, derived from the ticket. Used as the dedup key because
   * it survives the device's address changing, while the ticket string does
   * not. */
  nodeId: string;
  /** ISO 8601, or `null` if this device has never successfully connected. */
  lastConnectedAt: string | null;
  /** Last observed reachability. Not a live probe - set by whoever manages
   * the actual connection when it transitions. */
  lastKnownOnline: boolean;
}

export type PairedDeviceInput = Pick<PairedDevice, 'name' | 'ticket' | 'nodeId'>;

function blank(field: string): never {
  throw new Error(`paired device ${field} must not be blank`);
}

function normalize(input: PairedDeviceInput): PairedDeviceInput {
  const name = input.name.trim();
  const ticket = input.ticket.trim();
  const nodeId = input.nodeId.trim();
  if (name === '') blank('name');
  if (ticket === '') blank('ticket');
  if (nodeId === '') blank('nodeId');
  return { name, ticket, nodeId };
}

/** Holds one `PairedDevice` per `nodeId`. Insertion order is preserved for
 * `list()` so the device list UI does not reshuffle itself on every render. */
export class PairedDeviceStore {
  private devices = new Map<string, PairedDevice>();

  /** Adds a new device, or updates `name`/`ticket` if `nodeId` is already
   * known. Re-pairing an existing device (doc 5.2's reconnection case) must
   * not reset `lastConnectedAt`/`lastKnownOnline` - those describe the device,
   * not the ticket used to find it. */
  upsert(input: PairedDeviceInput): PairedDevice {
    const { name, ticket, nodeId } = normalize(input);
    const existing = this.devices.get(nodeId);
    const device: PairedDevice = existing
      ? { ...existing, name, ticket }
      : { name, ticket, nodeId, lastConnectedAt: null, lastKnownOnline: false };
    this.devices.set(nodeId, device);
    return { ...device };
  }

  remove(nodeId: string): boolean {
    return this.devices.delete(nodeId);
  }

  get(nodeId: string): PairedDevice | undefined {
    const device = this.devices.get(nodeId);
    return device ? { ...device } : undefined;
  }

  has(nodeId: string): boolean {
    return this.devices.has(nodeId);
  }

  list(): PairedDevice[] {
    return [...this.devices.values()].map((d) => ({ ...d }));
  }

  count(): number {
    return this.devices.size;
  }

  /** Records a successful connection. A no-op for an unknown `nodeId` rather
   * than an error: the caller may be reporting a stale event for a device the
   * user has since removed. */
  markConnected(nodeId: string, atIso: string): void {
    const device = this.devices.get(nodeId);
    if (!device) return;
    device.lastConnectedAt = atIso;
    device.lastKnownOnline = true;
  }

  setOnline(nodeId: string, online: boolean): void {
    const device = this.devices.get(nodeId);
    if (!device) return;
    device.lastKnownOnline = online;
  }

  /** Plain-JSON snapshot, safe to hand to `saveData`. */
  toJSON(): PairedDevice[] {
    return this.list();
  }

  /** Rebuilds a store from a previously persisted snapshot. Malformed entries
   * (missing fields, wrong types - e.g. from a downgrade or hand-edited
   * `data.json`) are skipped rather than aborting the whole load, so one bad
   * record cannot make every paired device disappear. */
  static fromJSON(raw: unknown): PairedDeviceStore {
    const store = new PairedDeviceStore();
    if (!Array.isArray(raw)) return store;

    for (const entry of raw) {
      if (!isPlainObject(entry)) continue;
      const { name, ticket, nodeId, lastConnectedAt, lastKnownOnline } = entry as Record<string, unknown>;
      if (typeof name !== 'string' || typeof ticket !== 'string' || typeof nodeId !== 'string') continue;
      if (name.trim() === '' || ticket.trim() === '' || nodeId.trim() === '') continue;

      store.devices.set(nodeId, {
        name,
        ticket,
        nodeId,
        lastConnectedAt: typeof lastConnectedAt === 'string' ? lastConnectedAt : null,
        lastKnownOnline: lastKnownOnline === true,
      });
    }
    return store;
  }
}

function isPlainObject(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}
