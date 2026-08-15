import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';

import {
  copyPackagedRuntime,
  migrateEnabledPluginId,
  migrateLegacyPluginData,
} from './install-dev-package.js';

test('copyPackagedRuntime installs core files and native runtime dependencies', () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'termesh-package-install-'));
  const packageDir = path.join(tempDir, 'package');
  const targetDir = path.join(tempDir, 'target');

  try {
    fs.mkdirSync(path.join(packageDir, 'binaries'), { recursive: true });
    fs.mkdirSync(path.join(packageDir, 'node_modules', '@number0', 'iroh'), { recursive: true });
    fs.writeFileSync(path.join(packageDir, 'main.js'), 'module.exports = {};');
    fs.writeFileSync(path.join(packageDir, 'manifest.json'), '{}');
    fs.writeFileSync(path.join(packageDir, 'styles.css'), '');
    fs.writeFileSync(path.join(packageDir, 'binaries', 'termy-server'), 'binary');
    fs.writeFileSync(
      path.join(packageDir, 'node_modules', '@number0', 'iroh', 'package.json'),
      '{"name":"@number0/iroh"}',
    );
    fs.mkdirSync(targetDir, { recursive: true });

    copyPackagedRuntime(packageDir, targetDir);

    assert.equal(fs.readFileSync(path.join(targetDir, 'main.js'), 'utf8'), 'module.exports = {};');
    assert.equal(fs.readFileSync(path.join(targetDir, 'binaries', 'termy-server'), 'utf8'), 'binary');
    assert.equal(
      JSON.parse(fs.readFileSync(
        path.join(targetDir, 'node_modules', '@number0', 'iroh', 'package.json'),
        'utf8',
      )).name,
      '@number0/iroh',
    );
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});

test('copyPackagedRuntime rejects incomplete packages', () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'termesh-package-install-'));

  try {
    assert.throws(
      () => copyPackagedRuntime(tempDir, path.join(tempDir, 'target')),
      /Packaged runtime entry is missing: main\.js/,
    );
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});

test('migrateEnabledPluginId replaces an enabled legacy ID without duplicates', () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'termesh-enabled-plugins-'));
  const pluginsDir = path.join(tempDir, 'plugins');
  const enabledPluginsPath = path.join(tempDir, 'community-plugins.json');

  try {
    fs.mkdirSync(pluginsDir);
    fs.writeFileSync(enabledPluginsPath, JSON.stringify(['termy', 'hi-note', 'termesh']));

    assert.equal(migrateEnabledPluginId(pluginsDir), true);
    assert.deepEqual(JSON.parse(fs.readFileSync(enabledPluginsPath, 'utf8')), [
      'termy',
      'hi-note',
    ]);
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});

test('migrateEnabledPluginId leaves vaults without the legacy ID unchanged', () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'termesh-enabled-plugins-'));
  const pluginsDir = path.join(tempDir, 'plugins');
  const enabledPluginsPath = path.join(tempDir, 'community-plugins.json');

  try {
    fs.mkdirSync(pluginsDir);
    fs.writeFileSync(enabledPluginsPath, JSON.stringify(['hi-note']));

    assert.equal(migrateEnabledPluginId(pluginsDir), false);
    assert.deepEqual(JSON.parse(fs.readFileSync(enabledPluginsPath, 'utf8')), ['hi-note']);
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});

test('migrateLegacyPluginData preserves settings when changing plugin IDs', () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'termesh-plugin-data-'));
  const pluginsDir = path.join(tempDir, 'plugins');

  try {
    fs.mkdirSync(path.join(pluginsDir, 'termy'), { recursive: true });
    fs.mkdirSync(path.join(pluginsDir, 'termesh'), { recursive: true });
    fs.writeFileSync(path.join(pluginsDir, 'termesh', 'data.json'), '{"shell":"pwsh"}');

    assert.equal(migrateLegacyPluginData(pluginsDir), true);
    assert.deepEqual(
      JSON.parse(fs.readFileSync(path.join(pluginsDir, 'termy', 'data.json'), 'utf8')),
      { shell: 'pwsh' },
    );
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});

test('migrateLegacyPluginData does not overwrite existing Termy settings', () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'termesh-plugin-data-'));
  const pluginsDir = path.join(tempDir, 'plugins');

  try {
    fs.mkdirSync(path.join(pluginsDir, 'termy'), { recursive: true });
    fs.mkdirSync(path.join(pluginsDir, 'termesh'), { recursive: true });
    fs.writeFileSync(path.join(pluginsDir, 'termesh', 'data.json'), '{"shell":"legacy"}');
    fs.writeFileSync(path.join(pluginsDir, 'termy', 'data.json'), '{"shell":"current"}');

    assert.equal(migrateLegacyPluginData(pluginsDir), false);
    assert.deepEqual(
      JSON.parse(fs.readFileSync(path.join(pluginsDir, 'termy', 'data.json'), 'utf8')),
      { shell: 'current' },
    );
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});