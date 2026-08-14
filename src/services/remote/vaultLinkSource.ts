/**
 * Adapts Obsidian's Vault and MetadataCache to the `LinkSource` the collector
 * needs (doc 10.1).
 *
 * Kept apart from `noteCollector.ts` so the collection rules stay testable
 * without an Obsidian runtime.
 */

import type { App, TFile } from 'obsidian';

import type { LinkSource, ResolvedLink } from './noteCollector.ts';

/**
 * Reads links from the note's cached metadata.
 *
 * `resolvedLinks` is not used directly: it is keyed by resolved path and loses
 * both the original link text and the ordering, and the collector needs the raw
 * text to tell an unresolved attachment from an unresolved note link.
 */
export function createVaultLinkSource(app: App, note: TFile): LinkSource {
  const cache = app.metadataCache.getFileCache(note);

  const raws: string[] = [];
  for (const link of cache?.links ?? []) raws.push(link.link);
  for (const embed of cache?.embeds ?? []) raws.push(embed.link);

  const links: ResolvedLink[] = raws.map((raw) => {
    // Strip a heading or block reference before resolving; "img.png#anchor"
    // still points at img.png.
    const target = raw.split('#')[0].split('|')[0].trim();
    const dest = target === '' ? null : app.metadataCache.getFirstLinkpathDest(target, note.path);
    return { raw, resolved: dest ? dest.path : null };
  });

  return {
    rootNotePath: note.path,
    links: () => links,
    sizeOf: (path: string): number | null => {
      const file = app.vault.getFileByPath(path);
      return file ? file.stat.size : null;
    },
  };
}

/**
 * Same as `createVaultLinkSource`, but looks the note up by vault-relative
 * path instead of taking a `TFile` directly — used by the v3.1 recursive
 * collector to build a `LinkSource` for each note it discovers while
 * walking links, not just the one the user triggered the send from.
 */
export function createVaultLinkSourceForPath(app: App, path: string): LinkSource | null {
  const file = app.vault.getFileByPath(path);
  return file ? createVaultLinkSource(app, file) : null;
}

/** Reads a vault file's bytes for sending. */
export async function readVaultFile(app: App, path: string): Promise<Uint8Array> {
  const file = app.vault.getFileByPath(path);
  if (!file) {
    throw new Error(`${path} disappeared from the vault before it could be sent`);
  }
  const buffer = await app.vault.readBinary(file);
  return new Uint8Array(buffer);
}
