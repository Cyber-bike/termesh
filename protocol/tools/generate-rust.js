'use strict';

/**
 * Bundles the schemas and runs typify to produce generated/rust/src/messages.rs.
 * Output is checked in; CI regenerates and fails on a diff.
 */

const fs = require('fs');
const path = require('path');
const { execFileSync } = require('child_process');

const ROOT = path.join(__dirname, '..');
const BUNDLE = path.join(ROOT, 'generated', 'protocol.bundle.schema.json');
const OUT = path.join(ROOT, 'generated', 'rust', 'src', 'messages.rs');

execFileSync(process.execPath, [path.join(__dirname, 'bundle-schema.js')], { stdio: 'inherit' });

const cargo = process.env.CARGO || path.join(process.env.HOME || '/root', '.cargo', 'bin', 'cargo');
if (!fs.existsSync(cargo)) {
  console.error(`cargo not found at ${cargo}. Install the toolchain pinned in rust-toolchain.toml.`);
  process.exit(1);
}

try {
  execFileSync(cargo, ['typify', BUNDLE, '-o', OUT], { stdio: 'pipe', encoding: 'utf8' });
} catch (err) {
  console.error(err.stdout || '');
  console.error(err.stderr || err.message);
  console.error('\ncargo typify failed. Install it with `cargo install cargo-typify --locked`.');
  process.exit(1);
}

const banner = `// GENERATED FILE - DO NOT EDIT.
// Source: protocol/schema/ via protocol/generated/protocol.bundle.schema.json.
// Regenerate with \`npm run generate:rust\` in protocol/.

`;
fs.writeFileSync(OUT, banner + fs.readFileSync(OUT, 'utf8'));

const lines = fs.readFileSync(OUT, 'utf8').split('\n').length;
console.log(`Wrote ${path.relative(ROOT, OUT)} (${lines} lines)`);
