/**
 * Send orchestration for the v3.1 right-click / note-toolbar entries
 * (`输出文档/v3.1 需求文档采集.md` §2/§3): collect the triggering note plus
 * every note it links to, recursively, then hand the flat file list to the
 * same transfer pipeline `handleRemoteDrop` already uses.
 *
 * A thin Obsidian-facing wrapper around `collectRecursive` — not unit
 * tested directly, same convention as `vaultLinkSource.ts` and
 * `directoryTreeDrop.ts` (needs a real `App`/device connection).
 */

import type { App, TFile } from 'obsidian';

import { checkQuotas } from './noteCollector.ts';
import { collectRecursive, type SkippedNote } from './noteCollectorRecursive.ts';
import { createVaultLinkSource, createVaultLinkSourceForPath, readVaultFile } from './vaultLinkSource.ts';
import type { DeviceConnectionManager } from './deviceConnections.ts';

export interface SendNoteRecursivelyResult {
  success: boolean;
  message?: string;
  skippedNotes: SkippedNote[];
}

export async function sendNoteRecursively(
  app: App,
  file: TFile,
  nodeId: string,
  connections: DeviceConnectionManager,
  targetPath: string,
): Promise<SendNoteRecursivelyResult> {
  const collected = collectRecursive(createVaultLinkSource(app, file), (path) =>
    createVaultLinkSourceForPath(app, path)
  );
  if (!collected.ok) {
    return { success: false, message: collected.error ?? 'Unable to collect note', skippedNotes: [] };
  }

  const quota = checkQuotas(collected.files);
  if (!quota.ok) {
    return { success: false, message: quota.error ?? 'Transfer quota exceeded', skippedNotes: collected.skippedNotes };
  }

  const outcome = await connections
    .createTransferSender(nodeId, crypto.randomUUID(), collected.files, (path) => readVaultFile(app, path), null, targetPath)
    .run();

  if (!outcome.success) {
    return { success: false, message: outcome.message || 'Transfer failed', skippedNotes: collected.skippedNotes };
  }

  return { success: true, skippedNotes: collected.skippedNotes };
}
