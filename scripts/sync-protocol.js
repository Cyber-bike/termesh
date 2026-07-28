#!/usr/bin/env node
/**
 * Copies the generated protocol types into the plugin source tree (doc 8.1).
 *
 * They are generated in protocol/ and consumed here; keeping a copy in src/
 * means esbuild does not have to reach outside the plugin root, and a stale copy
 * is caught by `--check` in CI rather than by a confusing type error.
 */
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const from = path.join(here, '..', 'protocol', 'generated', 'typescript', 'messages.ts');
const to = path.join(here, '..', 'src', 'protocol', 'generated', 'messages.ts');

if (!fs.existsSync(from)) {
  console.error(`missing ${from}; run \`npm run generate\` in protocol/ first`);
  process.exit(1);
}

const source = fs.readFileSync(from, 'utf8');

if (process.argv.includes('--check')) {
  const current = fs.existsSync(to) ? fs.readFileSync(to, 'utf8') : '';
  if (current !== source) {
    console.error('src/protocol/generated/messages.ts is stale; run `node scripts/sync-protocol.js`');
    process.exit(1);
  }
  console.log('generated protocol types are up to date');
  process.exit(0);
}

fs.mkdirSync(path.dirname(to), { recursive: true });
fs.writeFileSync(to, source);
console.log(`synced ${path.relative(process.cwd(), to)}`);
