/**
 * Vault-relative path pre-check (doc 10.3).
 *
 * The agent repeats every one of these rules and its answer is authoritative;
 * doing them here as well turns a doomed transfer into an immediate, specific
 * message instead of a round trip that fails halfway.
 *
 * Windows rules are applied regardless of the sending platform, because the
 * target may be Windows even when the control machine is not.
 */

export type PathRejection =
  | 'empty'
  | 'too-long'
  | 'nul-byte'
  | 'backslash'
  | 'absolute'
  | 'traversal'
  | 'empty-segment'
  | 'windows-reserved-name'
  | 'windows-illegal-char'
  | 'windows-trailing-dot-or-space';

export interface PathCheck {
  ok: boolean;
  reason?: PathRejection;
  detail?: string;
}

const MAX_PATH_BYTES = 1024;

const WINDOWS_RESERVED = new Set([
  'CON', 'PRN', 'AUX', 'NUL',
  'COM1', 'COM2', 'COM3', 'COM4', 'COM5', 'COM6', 'COM7', 'COM8', 'COM9',
  'LPT1', 'LPT2', 'LPT3', 'LPT4', 'LPT5', 'LPT6', 'LPT7', 'LPT8', 'LPT9',
]);

const reject = (reason: PathRejection, detail?: string): PathCheck => ({ ok: false, reason, detail });

export function utf8Length(value: string): number {
  return new TextEncoder().encode(value).length;
}

export function checkRelativePath(path: string): PathCheck {
  if (path.length === 0) return reject('empty');
  if (utf8Length(path) > MAX_PATH_BYTES) return reject('too-long');
  if (path.includes('\0')) return reject('nul-byte');
  if (path.includes('\\')) return reject('backslash');
  if (path.startsWith('/')) return reject('absolute');
  if (/^[A-Za-z]:/.test(path)) return reject('absolute');

  for (const segment of path.split('/')) {
    if (segment === '') return reject('empty-segment');
    if (segment === '..') return reject('traversal');
    if (segment === '.') return reject('empty-segment');

    if (segment.endsWith('.') || segment.endsWith(' ')) {
      return reject('windows-trailing-dot-or-space', segment);
    }
    if (/[<>:"|?*]/.test(segment)) return reject('windows-illegal-char', segment);
    // eslint-disable-next-line no-control-regex -- Intentionally matches ASCII control chars (U+0000..U+001F) to reject Windows-illegal path segments.
    if (/[\u0000-\u001f]/.test(segment)) return reject('windows-illegal-char', segment);

    const stem = segment.split('.')[0].toUpperCase();
    if (WINDOWS_RESERVED.has(stem)) return reject('windows-reserved-name', stem);
  }

  return { ok: true };
}

/** A message the UI can show as-is. */
export function describeRejection(path: string, check: PathCheck): string {
  switch (check.reason) {
    case 'empty':
      return 'A file path is empty.';
    case 'too-long':
      return `"${path}" is longer than 1024 bytes.`;
    case 'nul-byte':
      return `"${path}" contains a NUL byte.`;
    case 'backslash':
      return `"${path}" contains a backslash; only "/" separates vault paths.`;
    case 'absolute':
      return `"${path}" is an absolute path.`;
    case 'traversal':
      return `"${path}" tries to climb out of the vault.`;
    case 'empty-segment':
      return `"${path}" has an empty or "." path segment.`;
    case 'windows-reserved-name':
      return `"${path}" uses "${check.detail}", a reserved device name on Windows.`;
    case 'windows-illegal-char':
      return `"${path}" contains a character Windows does not allow in "${check.detail}".`;
    case 'windows-trailing-dot-or-space':
      return `"${check.detail}" ends with a dot or space, which Windows silently strips.`;
    default:
      return `"${path}" cannot be written safely on the target.`;
  }
}

/** Vault paths already use '/', but Obsidian hands back a few shapes worth normalising. */
export function normalizeVaultPath(path: string): string {
  return path.replace(/^\.\//, '').replace(/\/{2,}/g, '/');
}
