'use strict';

/**
 * Compile-time contract check: every valid fixture is assigned to the generated
 * type for its wire type. If the schemas, the generator and the fixtures ever
 * drift apart, tsc fails here rather than at runtime in the plugin.
 *
 * Also asserts a couple of negatives with @ts-expect-error, so a generated type
 * that degenerates to `any` or `unknown` would be caught too.
 */

const fs = require('fs');
const path = require('path');
const { execFileSync } = require('child_process');

const ROOT = path.join(__dirname, '..');
const VALID_DIR = path.join(ROOT, 'fixtures', 'valid');
const TMP = path.join(ROOT, '.typecheck');

const files = fs.readdirSync(VALID_DIR).filter((f) => f.endsWith('.json')).sort();

const lines = [
  "import type { ControlMessageByType } from '../generated/typescript/messages';",
  ''
];

// The fixture is inlined as an object literal rather than imported. A JSON
// module import widens 1 to number and 'terminal.open' to string, which would
// fail against the literal types on every message; a literal with a contextual
// type keeps its literal types and still gets excess-property checking.
files.forEach((file, i) => {
  const fixture = JSON.parse(fs.readFileSync(path.join(VALID_DIR, file), 'utf8'));
  lines.push(`// ${file}`);
  lines.push(
    `const check${i}: ControlMessageByType['${fixture.type}'] = ${JSON.stringify(fixture, null, 4)};`
  );
  lines.push(`void check${i};`);
  lines.push('');
});

// Negatives: each must fail exactly where the directive sits, otherwise tsc
// reports an unused @ts-expect-error and this script fails.
lines.push(
  "const bad1: ControlMessageByType['terminal.open'] = {",
  '    protocolVersion: 1,',
  "    type: 'terminal.open',",
  "    requestId: 'b7c1a2d3-4e5f-4a6b-8c9d-0e1f2a3b4c5d',",
  "    deviceId: '3d594650-3436-4c7a-9a15-9b5c3f0f4a11',",
  '    sessionId: null,',
  '    // @ts-expect-error cols must be a number, not a string',
  "    payload: { cols: '120', rows: 30 }",
  '};',
  'void bad1;',
  '',
  "const bad2: ControlMessageByType['terminal.open'] = {",
  '    protocolVersion: 1,',
  "    type: 'terminal.open',",
  "    requestId: 'b7c1a2d3-4e5f-4a6b-8c9d-0e1f2a3b4c5d',",
  "    deviceId: '3d594650-3436-4c7a-9a15-9b5c3f0f4a11',",
  '    // @ts-expect-error terminal.open must not carry a sessionId',
  "    sessionId: 'c9e0f1a2-b3c4-4d5e-9f60-718293a4b5c6',",
  '    payload: { cols: 120, rows: 30 }',
  '};',
  'void bad2;',
  ''
);

fs.mkdirSync(TMP, { recursive: true });
const entry = path.join(TMP, 'fixtures.ts');
fs.writeFileSync(entry, lines.join('\n'));

try {
  const tscExecutable = path.join(ROOT, 'node_modules', 'typescript', 'bin', 'tsc');
  execFileSync(
    process.execPath,
    [
      tscExecutable,
      '--noEmit', '--strict', '--target', 'ES2022', '--module', 'ESNext',
      '--moduleResolution', 'bundler', entry
    ],
    { stdio: 'pipe', encoding: 'utf8' }
  );
  console.log(`Typechecked ${files.length} fixtures against the generated types`);
} catch (err) {
  console.error(err.stdout || err.message);
  console.error('\nGenerated types do not match the fixtures');
  process.exit(1);
} finally {
  fs.rmSync(TMP, { recursive: true, force: true });
}
