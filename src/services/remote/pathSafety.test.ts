import * as assert from 'node:assert/strict';
import * as fs from 'node:fs';
import * as path from 'node:path';
import test from 'node:test';

import { checkRelativePath, describeRejection, normalizeVaultPath } from './pathSafety.ts';

test('ordinary vault paths pass', () => {
  for (const candidate of [
    'a.md',
    'notes/demo.md',
    'assets/img/diagram.png',
    'a b/c d.md',
    '中文/笔记.md',
    'console.md',
    'nullable.md',
  ]) {
    assert.equal(checkRelativePath(candidate).ok, true, candidate);
  }
});

test('structural attacks are refused', () => {
  const cases: Array<[string, string]> = [
    ['', 'empty'],
    ['/etc/passwd', 'absolute'],
    ['C:/Windows/system.ini', 'absolute'],
    ['../escape.md', 'traversal'],
    ['notes/../../etc/shadow', 'traversal'],
    ['notes//demo.md', 'empty-segment'],
    ['notes/./demo.md', 'empty-segment'],
    ['notes\\demo.md', 'backslash'],
    ['a\0b.md', 'nul-byte'],
  ];

  for (const [candidate, reason] of cases) {
    const check = checkRelativePath(candidate);
    assert.equal(check.ok, false, candidate);
    assert.equal(check.reason, reason, candidate);
  }
});

test('windows rules apply even when sending from elsewhere', () => {
  // The target may be Windows regardless of what the control machine runs.
  for (const [candidate, reason] of [
    ['CON.txt', 'windows-reserved-name'],
    ['nul', 'windows-reserved-name'],
    ['a/COM1.md', 'windows-reserved-name'],
    ['trailing.', 'windows-trailing-dot-or-space'],
    ['trailing ', 'windows-trailing-dot-or-space'],
    ['what?.md', 'windows-illegal-char'],
    // A leading "x:" is a drive-relative path on Windows, so it is classified as
    // absolute rather than as an illegal character. A colon deeper in the path
    // has no such meaning and is just forbidden.
    ['a:b.md', 'absolute'],
    ['notes/a:b.md', 'windows-illegal-char'],
  ] as Array<[string, string]>) {
    const check = checkRelativePath(candidate);
    assert.equal(check.ok, false, candidate);
    assert.equal(check.reason, reason, candidate);
  }
});

test('a control character is caught rather than matching everything', () => {
  // Guards a real defect: the control-class regex was once mangled into a
  // negated class, which rejected every path instead of only bad ones.
  assert.equal(checkRelativePath('bad\u0001name.md').ok, false);
  assert.equal(checkRelativePath('plain-name.md').ok, true);
});

test('a path at the byte limit is accepted and one over is not', () => {
  const justUnder = `${'x'.repeat(1020)}.md`;
  assert.equal(checkRelativePath(justUnder).ok, true);
  assert.equal(checkRelativePath('x'.repeat(1025)).reason, 'too-long');

  // The limit counts UTF-8 bytes, not characters.
  assert.equal(checkRelativePath('中'.repeat(400)).reason, 'too-long');
});

test('rejections explain themselves', () => {
  const check = checkRelativePath('../escape.md');
  const message = describeRejection('../escape.md', check);
  assert.match(message, /climb out/);
  assert.ok(message.includes('../escape.md'));
});

test('vault paths are normalised', () => {
  assert.equal(normalizeVaultPath('./notes/demo.md'), 'notes/demo.md');
  assert.equal(normalizeVaultPath('notes//demo.md'), 'notes/demo.md');
  assert.equal(normalizeVaultPath('notes/demo.md'), 'notes/demo.md');
});

test('the plugin agrees with the shared invalid path fixtures', () => {
  // Same corpus the relay and agent run, so a rule cannot be relaxed on one end
  // without the others noticing.
  const dir = path.join(process.cwd(), 'protocol', 'fixtures', 'invalid');
  // Only the fixtures that violate the *shape* of a path. Fixtures like
  // transfer-start-duplicate-path.json break a cross-field rule instead, which
  // the manifest pre-check catches rather than this function.
  const pathFixtures = [
    'transfer-start-path-traversal.json',
    'transfer-start-nested-traversal.json',
    'transfer-start-absolute-path.json',
    'transfer-start-drive-letter.json',
    'transfer-start-backslash-path.json',
  ];

  for (const file of pathFixtures) {
    assert.ok(fs.existsSync(path.join(dir, file)), `missing shared fixture ${file}`);
  }

  for (const file of pathFixtures) {
    const fixture = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
    const entries = fixture?.payload?.entries ?? [];
    const anyRejected = entries.some(
      (entry: { relativePath: string }) => !checkRelativePath(entry.relativePath).ok
    );
    assert.equal(anyRejected, true, `${file} should contain a path the plugin refuses`);
  }
});
