/**
 * Collects the note and its direct attachments (doc 10.1).
 *
 * Deliberately independent of Obsidian's classes: it takes a small `LinkSource`
 * view of the vault so the rules can be tested without a running app. The
 * adapter that fills it in lives in `noteCollectorObsidian.ts`.
 */

import { checkRelativePath, describeRejection, normalizeVaultPath } from './pathSafety.ts';

/** Extensions treated as attachments. Anything else that fails to resolve is ignored. */
const ATTACHMENT_EXTENSIONS = new Set([
  'png', 'jpg', 'jpeg', 'gif', 'webp', 'svg', 'bmp', 'ico', 'avif',
  'pdf',
  'mp3', 'wav', 'ogg', 'm4a', 'flac', '3gp',
  'mp4', 'webm', 'mov', 'mkv',
  'zip', 'gz', 'tar', '7z', 'rar',
  'txt', 'csv', 'json', 'yaml', 'yml', 'toml',
  'docx', 'xlsx', 'pptx',
  'canvas', 'base',
]);

export interface CollectedFile {
  index: number;
  relativePath: string;
  size: number;
}

export interface LinkSource {
  /** Vault-relative path of the note being sent. */
  rootNotePath: string;
  /** Byte size of a vault file, or null when it does not exist. */
  sizeOf(path: string): number | null;
  /** Links found in the root note, in document order, already resolved where possible. */
  links(): ResolvedLink[];
}

export interface ResolvedLink {
  /** The raw link target as written in the note. */
  raw: string;
  /** Vault-relative path when Obsidian could resolve it, otherwise null. */
  resolved: string | null;
}

export interface CollectResult {
  ok: boolean;
  files: CollectedFile[];
  /** Populated when ok is false. */
  error?: string;
  /** Links that were skipped, for the debug log. */
  skipped: string[];
}

export function isExternal(raw: string): boolean {
  return /^[a-z][a-z0-9+.-]*:/i.test(raw) && !/^[a-z]:[\\/]/i.test(raw);
}

export function extensionOf(target: string): string {
  const withoutFragment = target.split('#')[0].split('?')[0];
  const base = withoutFragment.slice(withoutFragment.lastIndexOf('/') + 1);
  const dot = base.lastIndexOf('.');
  return dot <= 0 ? '' : base.slice(dot + 1).toLowerCase();
}

/**
 * True when a link looks like it points at an attachment.
 *
 * This is what decides whether an unresolved link is fatal. Obsidian vaults
 * routinely contain links to notes that do not exist yet, so treating every
 * unresolved link as an error would make most real notes untransferable (doc
 * 10.1). Only a link that clearly names an attachment is worth failing over.
 */
export function looksLikeAttachment(raw: string): boolean {
  const extension = extensionOf(raw);
  return extension !== '' && extension !== 'md' && ATTACHMENT_EXTENSIONS.has(extension);
}

export function collect(source: LinkSource): CollectResult {
  const skipped: string[] = [];
  const rootPath = normalizeVaultPath(source.rootNotePath);

  const rootCheck = checkRelativePath(rootPath);
  if (!rootCheck.ok) {
    return { ok: false, files: [], skipped, error: describeRejection(rootPath, rootCheck) };
  }

  const rootSize = source.sizeOf(rootPath);
  if (rootSize === null) {
    return { ok: false, files: [], skipped, error: `${rootPath} no longer exists in the vault.` };
  }

  const files: CollectedFile[] = [{ index: 0, relativePath: rootPath, size: rootSize }];
  const seen = new Set([rootPath]);

  for (const link of source.links()) {
    if (isExternal(link.raw)) {
      skipped.push(`${link.raw} (external)`);
      continue;
    }

    if (link.resolved === null) {
      if (looksLikeAttachment(link.raw)) {
        // Doc 10.1: an attachment that cannot be found is fatal, because sending
        // the note without it would silently produce a broken document.
        return {
          ok: false,
          files: [],
          skipped,
          error: `Attachment "${link.raw}" could not be found in the vault.`,
        };
      }
      // An unresolved link to a note is ordinary in Obsidian; ignore it.
      skipped.push(`${link.raw} (unresolved, not an attachment)`);
      continue;
    }

    const resolved = normalizeVaultPath(link.resolved);

    if (extensionOf(resolved) === 'md') {
      // Doc 10.1: links to other notes are not followed.
      skipped.push(`${resolved} (markdown, not recursed)`);
      continue;
    }

    if (seen.has(resolved)) continue;

    const check = checkRelativePath(resolved);
    if (!check.ok) {
      return { ok: false, files: [], skipped, error: describeRejection(resolved, check) };
    }

    const size = source.sizeOf(resolved);
    if (size === null) {
      return {
        ok: false,
        files: [],
        skipped,
        error: `Attachment "${resolved}" could not be read.`,
      };
    }

    seen.add(resolved);
    files.push({ index: files.length, relativePath: resolved, size });
  }

  return { ok: true, files, skipped };
}

/** Doc 4.12 and 8.4. */
export const MAX_FILES = 256;
export const MAX_FILE_BYTES = 64 * 1024 * 1024;
export const MAX_TRANSFER_BYTES = 256 * 1024 * 1024;

export function checkQuotas(files: CollectedFile[]): { ok: boolean; error?: string } {
  if (files.length > MAX_FILES) {
    return { ok: false, error: `This note pulls in ${files.length} files; the limit is ${MAX_FILES}.` };
  }

  for (const file of files) {
    if (file.size > MAX_FILE_BYTES) {
      return {
        ok: false,
        error: `"${file.relativePath}" is ${formatBytes(file.size)}; the per-file limit is 64 MiB.`,
      };
    }
  }

  const total = files.reduce((sum, file) => sum + file.size, 0);
  if (total > MAX_TRANSFER_BYTES) {
    return {
      ok: false,
      error: `The note and its attachments total ${formatBytes(total)}; the limit is 256 MiB.`,
    };
  }

  return { ok: true };
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KiB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MiB`;
}
