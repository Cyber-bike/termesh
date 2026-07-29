import assert from 'node:assert/strict';
import test from 'node:test';

import {
  CODEX_LAUNCH_COMMAND,
  DEFAULT_PRESET_SCRIPTS,
  DEFAULT_REMOTE_CONNECTION_SETTINGS,
  OPENCODE_LAUNCH_COMMAND,
  isContextAwarePresetScript,
  normalizeRemoteRelayUrl,
} from './settings.ts';

test('Codex built-in launcher starts Codex without prompt injection', () => {
  const codex = DEFAULT_PRESET_SCRIPTS.find((script) => script.id === 'codex');
  const launchAction = codex?.actions.find((action) => action.id === 'action-codex');

  assert.equal(CODEX_LAUNCH_COMMAND, 'codex');
  assert.equal(launchAction?.value, CODEX_LAUNCH_COMMAND);
});

test('OpenCode built-in launcher starts the IDE bridge client directly', () => {
  const openCode = DEFAULT_PRESET_SCRIPTS.find((script) => script.id === 'opencode');
  const launchAction = openCode?.actions.find((action) => action.id === 'action-opencode');

  assert.equal(OPENCODE_LAUNCH_COMMAND, 'opencode');
  assert.equal(launchAction?.value, OPENCODE_LAUNCH_COMMAND);
});

test('built-in workflow order keeps Claude Code, Codex, and OpenCode', () => {
  assert.deepEqual(
    DEFAULT_PRESET_SCRIPTS.map((script) => script.id),
    ['claude-code', 'codex', 'opencode'],
  );

  assert.equal(DEFAULT_PRESET_SCRIPTS[2]?.id, 'opencode');
});

test('built-in context-aware workflow marker covers all built-ins', () => {
  const contextAwareIds = DEFAULT_PRESET_SCRIPTS
    .filter((script) => isContextAwarePresetScript(script))
    .map((script) => script.id);

  assert.deepEqual(contextAwareIds, ['claude-code', 'codex', 'opencode']);
});

test('remote relay URL migrates the legacy download host', () => {
  assert.equal(DEFAULT_REMOTE_CONNECTION_SETTINGS.relayUrl, 'https://bjev.duckdns.org');
  assert.equal(
    normalizeRemoteRelayUrl('https://termy.changqiu.xyz'),
    DEFAULT_REMOTE_CONNECTION_SETTINGS.relayUrl,
  );
  assert.equal(normalizeRemoteRelayUrl('https://relay.example.com/base?ignored=1'), 'https://relay.example.com');
});
