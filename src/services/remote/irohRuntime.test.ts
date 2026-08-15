import * as assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';

import { createIrohLoader } from './irohRuntime.ts';
import type { IrohModule } from './irohStreams.ts';

test('loads iroh from the plugin-local node_modules directory', async () => {
  const expected = {} as IrohModule;
  let loadedPath = '';
  const loadIroh = createIrohLoader('C:\\vault\\.obsidian\\plugins\\termy', (modulePath) => {
    loadedPath = modulePath;
    return expected;
  });

  assert.equal(await loadIroh(), expected);
  assert.equal(
    loadedPath,
    path.join('C:\\vault\\.obsidian\\plugins\\termy', 'node_modules', '@number0', 'iroh'),
  );
});

test('reports a clear installation error when the native module is missing', async () => {
  const loadIroh = createIrohLoader('/vault/.obsidian/plugins/termy', () => {
    throw new Error('module not found');
  });

  await assert.rejects(loadIroh(), /无法加载远程终端原生模块.*平台完整包.*pnpm package.*module not found/);
});

test('installs and loads the runtime when the packaged module is missing', async () => {
  const expected = {} as IrohModule;
  const loadedPaths: string[] = [];
  let installCount = 0;
  const loadIroh = createIrohLoader('/vault/.obsidian/plugins/termesh', (modulePath) => {
    loadedPaths.push(modulePath);
    if (modulePath.endsWith(path.join('@number0', 'iroh'))) {
      throw new Error('module not found');
    }
    return expected;
  }, {
    version: '1.5.0',
    isOffline: () => false,
    installRuntime: () => {
      installCount += 1;
      return Promise.resolve({
        nativePath: '/runtime/iroh.node',
      });
    },
  });

  assert.equal(await loadIroh(), expected);
  assert.equal(await loadIroh(), expected);
  assert.equal(installCount, 1);
  assert.deepEqual(loadedPaths, [
    path.join('/vault/.obsidian/plugins/termesh', 'node_modules', '@number0', 'iroh'),
    '/runtime/iroh.node',
  ]);
});

test('does not install the runtime in offline mode', async () => {
  let installCalled = false;
  const loadIroh = createIrohLoader('/vault/.obsidian/plugins/termesh', () => {
    throw new Error('module not found');
  }, {
    version: '1.5.0',
    isOffline: () => true,
    installRuntime: () => {
      installCalled = true;
      return Promise.reject(new Error('unexpected install'));
    },
  });

  await assert.rejects(loadIroh(), /离线模式禁止自动下载/);
  assert.equal(installCalled, false);
});