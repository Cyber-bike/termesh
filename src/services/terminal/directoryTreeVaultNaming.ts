/**
 * Conflict naming for the "tree -> vault" copy direction (candidate doc
 * §6.4): unlike the existing "vault -> terminal cwd" direction, a same-name
 * conflict here must NOT silently overwrite — the target is the user's
 * knowledge base, not a disposable working directory. This module picks a
 * free name instead of prompting, which keeps the copy a single atomic
 * action; `directoryTreeDrop.ts` is the thing that actually decides whether
 * an existing note is worth risking.
 */

/**
 * Given a desired vault-relative path, returns the same path if free, or
 * the first "name (2).ext", "name (3).ext", ... that isn't taken.
 * `exists` is a pure predicate so this stays testable without a Vault.
 */
export function resolveUniqueVaultPath(desiredPath: string, exists: (path: string) => boolean): string {
  if (!exists(desiredPath)) return desiredPath;

  const lastSlash = desiredPath.lastIndexOf('/');
  const dir = lastSlash === -1 ? '' : desiredPath.slice(0, lastSlash + 1);
  const base = lastSlash === -1 ? desiredPath : desiredPath.slice(lastSlash + 1);

  const dotIndex = base.lastIndexOf('.');
  // A leading dot ("`.gitignore`") is a hidden-file marker, not an extension.
  const hasExtension = dotIndex > 0;
  const stem = hasExtension ? base.slice(0, dotIndex) : base;
  const extension = hasExtension ? base.slice(dotIndex) : '';

  for (let suffix = 2; suffix < 10_000; suffix += 1) {
    const candidate = `${dir}${stem} (${suffix})${extension}`;
    if (!exists(candidate)) return candidate;
  }
  throw new Error(`Could not find a free name for "${desiredPath}" after 9999 attempts`);
}
