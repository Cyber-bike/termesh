import * as assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';

import { IrohRuntimeInstaller, resolveIrohRuntimeAssetUrls } from './irohRuntimeInstaller.ts';

test('resolves npm CDN and GitHub fallback runtime URLs', () => {
  const urls = resolveIrohRuntimeAssetUrls({
    version: '1.5.0',
    platform: 'win32',
    arch: 'x64',
  });

  assert.equal(
    urls.nativeUrl,
    'https://unpkg.com/@number0/iroh-win32-x64-msvc@1.1.0/iroh.win32-x64-msvc.node',
  );
  assert.deepEqual(urls.fallbackUrls, [
    'https://cdn.jsdelivr.net/npm/@number0/iroh-win32-x64-msvc@1.1.0/iroh.win32-x64-msvc.node',
    'https://github.com/jiang-zhong-xi/Termy/releases/download/1.5.0/iroh-runtime-win32-x64.node',
  ]);
  assert.equal(urls.expectedHash, '4fef61d33fc9a903a21cb1a2ae154b1e9576e8e453343b295d7d4d387e44bbe1');
});

test('downloads verified runtime files and reuses the installed version', async (context) => {
  const pluginDir = fs.mkdtempSync(path.join(os.tmpdir(), 'termesh-iroh-'));
  context.after(() => fs.rmSync(pluginDir, { recursive: true, force: true }));

  const native = Buffer.from('native fixture');
  let requestCount = 0;
  const fetchAsset = (_url: string, onProgress?: (downloadedBytes: number, totalBytes: number) => void): Promise<Buffer> => {
    requestCount += 1;
    onProgress?.(native.length / 2, native.length);
    onProgress?.(native.length, native.length);
    return Promise.resolve(native);
  };
  const installer = new IrohRuntimeInstaller(
    pluginDir,
    '1.5.0',
    fetchAsset,
    undefined,
    fixtureResolver(native),
  );
  const progress: string[] = [];

  const installed = await installer.ensureInstalled((event) => {
    progress.push(`${event.stage}:${event.percent === undefined ? '' : Math.round(event.percent)}`);
  });
  assert.deepEqual(fs.readFileSync(installed.nativePath), native);
  assert.equal(requestCount, 1);
  assert.deepEqual(progress, [
    'downloading:0',
    'downloading:50',
    'downloading:100',
    'verifying:',
    'complete:',
  ]);

  await installer.ensureInstalled();
  assert.equal(requestCount, 1);
});

test('rejects a runtime with a mismatched checksum', async (context) => {
  const pluginDir = fs.mkdtempSync(path.join(os.tmpdir(), 'termesh-iroh-'));
  context.after(() => fs.rmSync(pluginDir, { recursive: true, force: true }));

  const fetchAsset = (): Promise<Buffer> => Promise.resolve(Buffer.from('content'));
  const installer = new IrohRuntimeInstaller(pluginDir, '1.5.0', fetchAsset);

  await assert.rejects(installer.ensureInstalled(), /checksum mismatch/);
  assert.equal(fs.existsSync(path.join(pluginDir, 'native', 'iroh', 'iroh-runtime.node')), false);
});

test('falls back after the primary network path fails', async (context) => {
  const pluginDir = fs.mkdtempSync(path.join(os.tmpdir(), 'termesh-iroh-'));
  context.after(() => fs.rmSync(pluginDir, { recursive: true, force: true }));

  const native = Buffer.from('native fixture');
  const primaryRequests: string[] = [];
  const installer = new IrohRuntimeInstaller(
    pluginDir,
    '1.5.0',
    (url, onProgress) => {
      primaryRequests.push(url);
      if (url.includes('primary.example')) {
        return Promise.reject(new Error('primary CDN unavailable'));
      }
      onProgress?.(native.length, native.length);
      return Promise.resolve(native);
    },
    undefined,
    fixtureResolver(native),
  );
  const stages: string[] = [];

  await installer.ensureInstalled((progress) => stages.push(progress.stage));

  assert.deepEqual(primaryRequests, [
    'https://primary.example/iroh.node',
    'https://fallback.example/iroh.node',
  ]);
  assert.deepEqual(stages, ['downloading', 'downloading', 'verifying', 'complete']);
});

function fixtureResolver(content: Buffer) {
  return () => ({
    nativeUrl: 'https://primary.example/iroh.node',
    fallbackUrls: ['https://fallback.example/iroh.node'],
    expectedHash: crypto.createHash('sha256').update(content).digest('hex'),
  });
}