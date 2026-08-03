import * as assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';

import { createIrohLoader } from './irohRuntime.ts';
import type { IrohModule } from './irohStreams.ts';

test('loads iroh from the plugin-local node_modules directory', async () => {
  const expected = {} as IrohModule;
  let loadedPath = '';
  const loadIroh = createIrohLoader('C:\\vault\\.obsidian\\plugins\\termesh', (modulePath) => {
    loadedPath = modulePath;
    return expected;
  });

  assert.equal(await loadIroh(), expected);
  assert.equal(
    loadedPath,
    path.join('C:\\vault\\.obsidian\\plugins\\termesh', 'node_modules', '@number0', 'iroh'),
  );
});

test('reports a clear installation error when the native module is missing', async () => {
  const loadIroh = createIrohLoader('/vault/.obsidian/plugins/termesh', () => {
    throw new Error('module not found');
  });

  await assert.rejects(loadIroh(), /无法加载远程终端原生模块.*完整安装包.*pnpm package.*module not found/);
});