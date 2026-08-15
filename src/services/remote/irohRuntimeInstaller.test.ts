import * as assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';

import { IrohRuntimeInstaller, resolveIrohRuntimeAssetUrls } from './irohRuntimeInstaller.ts';

test('resolves versioned platform runtime asset URLs', () => {
  const urls = resolveIrohRuntimeAssetUrls({
    version: '1.5.0',
    platform: 'win32',
    arch: 'x64',
  });

  assert.equal(
    urls.nativeUrl,
    'https://github.com/jiang-zhong-xi/Termy/releases/download/1.5.0/iroh-runtime-win32-x64.node',
  );
  assert.equal(`${urls.nativeUrl}.sha256`, urls.nativeChecksumUrl);
});

test('downloads verified runtime files and reuses the installed version', async (context) => {
  const pluginDir = fs.mkdtempSync(path.join(os.tmpdir(), 'termesh-iroh-'));
  context.after(() => fs.rmSync(pluginDir, { recursive: true, force: true }));

  const native = Buffer.from('native fixture');
  let requestCount = 0;
  const fetchAsset = (url: string): Promise<Buffer> => {
    requestCount += 1;
    return Promise.resolve(url.endsWith('.sha256') ? checksum(native) : native);
  };
  const installer = new IrohRuntimeInstaller(pluginDir, '1.5.0', fetchAsset);

  const installed = await installer.ensureInstalled();
  assert.deepEqual(fs.readFileSync(installed.nativePath), native);
  assert.equal(requestCount, 2);

  await installer.ensureInstalled();
  assert.equal(requestCount, 2);
});

test('rejects a runtime with a mismatched checksum', async (context) => {
  const pluginDir = fs.mkdtempSync(path.join(os.tmpdir(), 'termesh-iroh-'));
  context.after(() => fs.rmSync(pluginDir, { recursive: true, force: true }));

  const fetchAsset = (url: string): Promise<Buffer> => Promise.resolve(
    url.endsWith('.sha256') ? Buffer.from('0'.repeat(64)) : Buffer.from('content'),
  );
  const installer = new IrohRuntimeInstaller(pluginDir, '1.5.0', fetchAsset);

  await assert.rejects(installer.ensureInstalled(), /checksum mismatch/);
  assert.equal(fs.existsSync(path.join(pluginDir, 'native', 'iroh', 'iroh-runtime.node')), false);
});

function checksum(content: Buffer): Buffer {
  return Buffer.from(`${crypto.createHash('sha256').update(content).digest('hex')}  asset\n`);
}