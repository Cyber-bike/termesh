import * as assert from 'node:assert/strict';
import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import test from 'node:test';

import { LocalDirectoryTreeSource } from './directoryTreeSource.ts';

function makeTmpDir(): string {
  return fs.mkdtempSync(path.join(os.tmpdir(), 'termy-dirtree-'));
}

test('list() returns directories before files, alphabetically within each group', async () => {
  const dir = makeTmpDir();
  try {
    fs.writeFileSync(path.join(dir, 'b.txt'), '');
    fs.writeFileSync(path.join(dir, 'a.txt'), '');
    fs.mkdirSync(path.join(dir, 'zeta'));
    fs.mkdirSync(path.join(dir, 'alpha'));

    const source = new LocalDirectoryTreeSource(fs);
    const entries = await source.list(dir);

    assert.deepEqual(entries, [
      { name: 'alpha', isDirectory: true },
      { name: 'zeta', isDirectory: true },
      { name: 'a.txt', isDirectory: false },
      { name: 'b.txt', isDirectory: false },
    ]);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

test('list() rejects for a path that does not exist', async () => {
  const source = new LocalDirectoryTreeSource(fs);
  await assert.rejects(() => source.list(path.join(os.tmpdir(), 'termy-does-not-exist-xyz')));
});

test('watch() notifies on a filesystem change under the watched directory', async () => {
  const dir = makeTmpDir();
  try {
    const source = new LocalDirectoryTreeSource(fs);
    const changes: string[] = [];
    const disposable = source.watch(dir, (kind) => changes.push(kind));

    fs.writeFileSync(path.join(dir, 'new-file.txt'), '');

    await new Promise((resolve) => setTimeout(resolve, 400));
    disposable.dispose();

    assert.ok(changes.length >= 1, 'expected at least one change notification');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

test('watch() stops notifying after dispose()', async () => {
  const dir = makeTmpDir();
  try {
    const source = new LocalDirectoryTreeSource(fs);
    let count = 0;
    const disposable = source.watch(dir, () => { count += 1; });
    disposable.dispose();

    fs.writeFileSync(path.join(dir, 'after-dispose.txt'), '');
    await new Promise((resolve) => setTimeout(resolve, 400));

    assert.equal(count, 0);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

test('watch() on a directory that disappears does not throw', () => {
  const dir = makeTmpDir();
  fs.rmSync(dir, { recursive: true, force: true });

  const source = new LocalDirectoryTreeSource(fs);
  assert.doesNotThrow(() => {
    const disposable = source.watch(dir, () => {});
    disposable.dispose();
  });
});
