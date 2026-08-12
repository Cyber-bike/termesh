/**
 * Vault <-> filesystem copy bridge for the directory tree panel (candidate
 * doc §4.1 points 4/6, §6.4).
 *
 * Two directions, deliberately asymmetric (candidate doc §6.4):
 *  - vault -> fs (drop onto a tree node): same-name conflicts overwrite,
 *    same as the existing vault -> terminal-cwd drop. The target is a
 *    working directory, not the vault.
 *  - fs -> vault ("copy to vault" on a tree node, see the panel's context
 *    menu): same-name conflicts never overwrite; `resolveUniqueVaultPath`
 *    picks a free name instead, because the target is the user's
 *    knowledge base.
 *
 * Kept apart from the panel UI so the walking/copying logic isn't tangled
 * up with DOM/drag event handling. Not unit tested directly (needs a real
 * `App`/`Vault`, same as `vaultLinkSource.ts`); `resolveUniqueVaultPath`,
 * the one pure decision made here, has its own tests.
 */

import type { App, TFile, TFolder } from 'obsidian';
import { TFile as TFileClass, TFolder as TFolderClass } from 'obsidian';

import { checkRelativePath, normalizeVaultPath } from '../remote/pathSafety.ts';
import type { PulledFile } from '../remote/transferStreamPuller.ts';
import { resolveUniqueVaultPath } from './directoryTreeVaultNaming.ts';

interface MinimalFsPromises {
  mkdir(path: string, options: { recursive: true }): Promise<string | undefined>;
  writeFile(path: string, data: Uint8Array): Promise<void>;
  readdir(path: string, options: { withFileTypes: true }): Promise<Array<{ name: string; isDirectory(): boolean }>>;
  readFile(path: string): Promise<Buffer>;
}

export interface FsAccess {
  promises: MinimalFsPromises;
  join(...segments: string[]): string;
}

export interface CopyResult {
  fileCount: number;
}

/**
 * Copies a single vault file or an entire vault folder (recursively) into
 * `targetDir` on the local filesystem. Mirrors the folder's own name and
 * structure under `targetDir`, matching how the existing note-transfer
 * flow preserves vault-relative structure on the receiving end.
 */
export async function copyVaultEntryToDirectory(
  app: App,
  entry: TFile | TFolder,
  targetDir: string,
  fsAccess: FsAccess,
): Promise<CopyResult> {
  await fsAccess.promises.mkdir(targetDir, { recursive: true });

  if (entry instanceof TFileClass) {
    const bytes = await readVaultFileBytes(app, entry);
    await fsAccess.promises.writeFile(fsAccess.join(targetDir, entry.name), bytes);
    return { fileCount: 1 };
  }

  let fileCount = 0;
  const walk = async (folder: TFolder, destDir: string): Promise<void> => {
    await fsAccess.promises.mkdir(destDir, { recursive: true });
    for (const child of folder.children) {
      if (child instanceof TFolderClass) {
        await walk(child, fsAccess.join(destDir, child.name));
      } else if (child instanceof TFileClass) {
        const bytes = await readVaultFileBytes(app, child);
        await fsAccess.promises.writeFile(fsAccess.join(destDir, child.name), bytes);
        fileCount += 1;
      }
    }
  };
  await walk(entry, fsAccess.join(targetDir, entry.name));
  return { fileCount };
}

async function readVaultFileBytes(app: App, file: TFile): Promise<Uint8Array> {
  const buffer = await app.vault.readBinary(file);
  return new Uint8Array(buffer);
}

/**
 * Copies a filesystem file or directory (recursively) into the vault under
 * `targetVaultFolder`. Every path is checked with `checkRelativePath`
 * before writing (candidate doc §6.5: the tree itself isn't sandboxed to a
 * root, so the one thing worth re-validating on the way *into* the vault is
 * that the resulting vault path is well-formed) and de-conflicted with
 * `resolveUniqueVaultPath`.
 */
export async function copyFsEntryToVault(
  app: App,
  absolutePath: string,
  isDirectory: boolean,
  targetVaultFolder: string,
  fsAccess: FsAccess,
  baseName: string,
): Promise<CopyResult> {
  const exists = (vaultPath: string): boolean => app.vault.getAbstractFileByPath(vaultPath) !== null;

  if (!isDirectory) {
    const desired = normalizeVaultPath(joinVaultPath(targetVaultFolder, baseName));
    const check = checkRelativePath(desired);
    if (!check.ok) throw new Error(`Cannot copy to "${desired}": not a valid vault path`);
    const finalPath = resolveUniqueVaultPath(desired, exists);
    const bytes = await fsAccess.promises.readFile(absolutePath);
    await app.vault.createBinary(finalPath, toArrayBuffer(bytes));
    return { fileCount: 1 };
  }

  let fileCount = 0;
  const walk = async (srcDir: string, destVaultDir: string): Promise<void> => {
    const entries = await fsAccess.promises.readdir(srcDir, { withFileTypes: true });
    for (const entry of entries) {
      const srcPath = fsAccess.join(srcDir, entry.name);
      const destPath = normalizeVaultPath(joinVaultPath(destVaultDir, entry.name));
      if (entry.isDirectory()) {
        await walk(srcPath, destPath);
        continue;
      }
      const check = checkRelativePath(destPath);
      if (!check.ok) continue; // skip individually-invalid entries rather than abort the whole copy
      const finalPath = resolveUniqueVaultPath(destPath, exists);
      const bytes = await fsAccess.promises.readFile(srcPath);
      await app.vault.createBinary(finalPath, toArrayBuffer(bytes));
      fileCount += 1;
    }
  };
  await walk(absolutePath, joinVaultPath(targetVaultFolder, baseName));
  return { fileCount };
}

/**
 * Writes the files a `TransferStreamPuller` pulled from a remote device
 * into the vault under `targetVaultFolder` (candidate doc phase 2B: the
 * remote counterpart to `copyFsEntryToVault`'s local-fs walk). Each
 * `relativePath` already includes the pulled root's own name where
 * relevant (a single file is just its basename; a pulled directory's
 * entries are prefixed with that directory's name, mirroring how
 * `copyFsEntryToVault` roots its local walk at `targetVaultFolder/baseName`)
 * , so no separate `baseName` join is needed here.
 */
export async function writePulledFilesToVault(
  app: App,
  files: PulledFile[],
  targetVaultFolder: string,
): Promise<CopyResult> {
  const exists = (vaultPath: string): boolean => app.vault.getAbstractFileByPath(vaultPath) !== null;

  let fileCount = 0;
  for (const file of files) {
    const desired = normalizeVaultPath(joinVaultPath(targetVaultFolder, file.relativePath));
    const check = checkRelativePath(desired);
    if (!check.ok) continue; // skip individually-invalid entries rather than abort the whole copy
    const finalPath = resolveUniqueVaultPath(desired, exists);
    await app.vault.createBinary(finalPath, toArrayBuffer(file.data));
    fileCount += 1;
  }
  return { fileCount };
}

function joinVaultPath(folder: string, name: string): string {
  const trimmed = folder.replace(/\/+$/, '');
  return trimmed.length > 0 ? `${trimmed}/${name}` : name;
}

function toArrayBuffer(bytes: Uint8Array): ArrayBuffer {
  return bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength) as ArrayBuffer;
}
