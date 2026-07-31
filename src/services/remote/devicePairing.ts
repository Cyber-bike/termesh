/**
 * "添加设备" orchestration (v2.0 doc 5.2, UI flow doc 6.4).
 *
 * Glues the three pieces the paste box needs, in order: pre-validate the
 * pasted code (`connectionCode.ts`), parse it into an `EndpointId` via the
 * injected authoritative parser (the iroh binding, once A0 settles how it
 * loads), and record the device in the `PairedDeviceStore`. Kept free of
 * Obsidian imports so the whole flow is unit-testable with a fake parser.
 *
 * Every failure carries doc §13's `TICKET_INVALID`: whether the code fails
 * the cheap surface checks or the real parser rejects it, the user-facing
 * outcome is the same "连接码无效" state (doc 6.4 keeps them in the paste
 * box), just with a finer-grained `problem` for the message.
 */

import {
  checkConnectionCode,
  type ConnectionCodeParser,
  type ConnectionCodeProblem,
} from './connectionCode';
import type { PairedDevice, PairedDeviceStore } from './pairedDeviceStore';

export type PairDeviceResult =
  | { ok: true; device: PairedDevice }
  | { ok: false; code: 'TICKET_INVALID'; problem: ConnectionCodeProblem | 'unparseable' };

/**
 * `name` is the user-supplied label from the add-device UI; when blank, the
 * device is labelled with a short prefix of its id - the same convention as
 * the agent's own fingerprint display - so the list never shows an unnamed
 * row.
 */
export function pairDevice(
  store: PairedDeviceStore,
  parser: ConnectionCodeParser,
  rawCode: string,
  name: string,
): PairDeviceResult {
  const checked = checkConnectionCode(rawCode);
  if (!checked.ok) {
    return { ok: false, code: 'TICKET_INVALID', problem: checked.problem };
  }

  let nodeId: string;
  try {
    nodeId = parser(checked.normalized).nodeId;
  } catch {
    return { ok: false, code: 'TICKET_INVALID', problem: 'unparseable' };
  }

  const label = name.trim() || nodeId.slice(0, 10);
  const device = store.upsert({ name: label, ticket: checked.normalized, nodeId });
  return { ok: true, device };
}
