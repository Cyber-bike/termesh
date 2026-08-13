/**
 * Local directory-tree data source (candidate doc "目录树与双向文件传输" §4.1/§4.7).
 *
 * Phase 1 only: reads the machine Termy itself runs on via Node `fs`, no
 * new protocol or Rust module. A remote implementation of the same
 * `DirectoryTreeSource` interface is Phase 2 (needs a new `termy/fs/1`
 * ALPN, see the dev doc) and is not part of this module.
 */

import type { Disposable } from '../remote/transport.ts';
import { toDisposable } from '../remote/transport.ts';

export interface DirectoryEntry {
  name: string;
  isDirectory: boolean;
}

export type DirectoryChangeKind = 'created' | 'deleted' | 'renamed' | 'unknown';

export interface DirectoryTreeSource {
  /** Lists the direct children of `path` (not recursive). Rejects if `path` cannot be read. */
  list(path: string): Promise<DirectoryEntry[]>;
  /**
   * Subscribes to structural changes (create/delete/rename) directly under `path`.
   * Content-level changes inside files are out of scope (candidate doc §6.1).
   */
  watch(path: string, onChange: (kind: DirectoryChangeKind) => void): Disposable;
}

interface MinimalFsWatcher {
  close(): void;
}

interface MinimalFsPromises {
  readdir(path: string, options: { withFileTypes: true }): Promise<Array<{ name: string; isDirectory(): boolean }>>;
}

export interface MinimalFsModule {
  promises: MinimalFsPromises;
  watch(
    path: string,
    options: { persistent: boolean },
    listener: (eventType: string, filename: string | null) => void,
  ): MinimalFsWatcher;
}

const WATCH_DEBOUNCE_MS = 150;

function compareEntries(a: DirectoryEntry, b: DirectoryEntry): number {
  if (a.isDirectory !== b.isDirectory) return a.isDirectory ? -1 : 1;
  return a.name.localeCompare(b.name);
}

/**
 * Node `fs`-backed implementation. Takes the `fs` module as a constructor
 * argument (rather than importing it at module scope) so it can be
 * exercised with real Node `fs` in tests and with Electron's
 * `window.require('fs')` in the plugin, which is the same runtime either
 * way but keeps this module free of a top-level Node import (see the
 * `FsModule`/`PathModule` comment in `terminalView.ts`).
 */
export class LocalDirectoryTreeSource implements DirectoryTreeSource {
  constructor(private readonly fs: MinimalFsModule) {}

  async list(path: string): Promise<DirectoryEntry[]> {
    const dirents = await this.fs.promises.readdir(path, { withFileTypes: true });
    const entries = dirents.map((dirent) => ({
      name: dirent.name,
      isDirectory: dirent.isDirectory(),
    }));
    entries.sort(compareEntries);
    return entries;
  }

  watch(path: string, onChange: (kind: DirectoryChangeKind) => void): Disposable {
    let debounceTimer: number | null = null;
    let pendingKind: DirectoryChangeKind = 'unknown';

    const flush = (): void => {
      debounceTimer = null;
      onChange(pendingKind);
      pendingKind = 'unknown';
    };

    const scheduleFlush = (kind: DirectoryChangeKind): void => {
      pendingKind = pendingKind === 'unknown' ? kind : pendingKind;
      if (debounceTimer) return;
      debounceTimer = window.setTimeout(flush, WATCH_DEBOUNCE_MS);
    };

    let watcher: MinimalFsWatcher | null = null;
    try {
      watcher = this.fs.watch(path, { persistent: false }, (eventType) => {
        scheduleFlush(eventType === 'rename' ? 'renamed' : 'unknown');
      });
    } catch {
      // Directory may have been removed or be unreadable between list() and watch();
      // the panel treats "no further updates" as acceptable rather than throwing.
      watcher = null;
    }

    return toDisposable(() => {
      if (debounceTimer) window.clearTimeout(debounceTimer);
      watcher?.close();
    });
  }
}
