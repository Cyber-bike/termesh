import * as assert from 'node:assert/strict';
import test from 'node:test';

import { buildDeviceTerminalTitle } from './deviceTerminalTitle.ts';

test('combines the device and note names', () => {
  assert.equal(buildDeviceTerminalTitle('Build server', 'Release notes', 'Terminal'), 'Build server · Release notes');
});

test('uses the terminal fallback when no note is active', () => {
  assert.equal(buildDeviceTerminalTitle('Build server', null, 'Terminal'), 'Build server · Terminal');
});

test('trims names and preserves Unicode', () => {
  assert.equal(buildDeviceTerminalTitle('  本机  ', '  路径示例  ', 'Terminal'), '本机 · 路径示例');
});

test('uses the fallback for a blank device name', () => {
  assert.equal(buildDeviceTerminalTitle('  ', 'Demo', 'Terminal'), 'Terminal · Demo');
});