'use strict';

/**
 * Bundles the split schemas into one self-contained Draft 2020-12 document.
 *
 * typify takes a single file and does not follow file-relative $refs, so the
 * envelope is merged into each message (same flattening the TS generator does)
 * and every common $def is inlined under $defs. The bundle is a build artifact,
 * not a source of truth - schema/ stays authoritative.
 */

const fs = require('fs');
const path = require('path');

const ROOT = path.join(__dirname, '..');
const SCHEMA_DIR = path.join(ROOT, 'schema');
const MESSAGE_DIR = path.join(SCHEMA_DIR, 'messages');
const OUT = path.join(ROOT, 'generated', 'protocol.bundle.schema.json');

const readJson = (f) => JSON.parse(fs.readFileSync(f, 'utf8'));

const pascal = (s) =>
  s.replace(/[.\-_](\w)/g, (_, c) => c.toUpperCase()).replace(/^(\w)/, (_, c) => c.toUpperCase());

const common = readJson(path.join(SCHEMA_DIR, 'common.schema.json'));
const envelope = readJson(path.join(SCHEMA_DIR, 'control-envelope.schema.json'));

/**
 * Rewrite external refs into local #/$defs/... refs, and turn `const: X` into
 * `enum: [X]`.
 *
 * The const rewrite matters: typify maps `const` to serde_json::Value, so the
 * generated ControlMessage - which serde derives as #[serde(untagged)] - would
 * have no usable discriminant and could deserialize a message into the wrong
 * variant whenever two shapes overlap. A single-element enum is semantically
 * identical in JSON Schema but makes typify emit a real unit type, which both
 * pins the discriminant and makes untagged matching deterministic.
 */
function localiseRefs(node) {
  if (Array.isArray(node)) return node.map(localiseRefs);
  if (node && typeof node === 'object') {
    const out = {};
    for (const [k, v] of Object.entries(node)) {
      if (k === '$ref' && typeof v === 'string' && v.includes('common.schema.json#/$defs/')) {
        out[k] = `#/$defs/${v.slice(v.lastIndexOf('/') + 1)}`;
      } else if (k === 'const') {
        out.enum = [v];
      } else {
        out[k] = localiseRefs(v);
      }
    }
    return out;
  }
  return node;
}

const defs = {};
for (const [name, def] of Object.entries(common.$defs)) {
  defs[name] = { title: name, ...localiseRefs(def) };
}

const messageFiles = fs.readdirSync(MESSAGE_DIR).filter((f) => f.endsWith('.schema.json')).sort();
const messageNames = [];

for (const file of messageFiles) {
  const schema = readJson(path.join(MESSAGE_DIR, file));
  const wireType = schema.properties.type.const;
  const name = pascal(wireType) + 'Message';

  const merged = localiseRefs({
    type: 'object',
    description: schema.description,
    properties: { ...envelope.properties, ...schema.properties },
    required: envelope.required,
    additionalProperties: false
  });

  defs[name] = { title: name, ...merged };
  messageNames.push(name);
}

const bundle = {
  $schema: 'https://json-schema.org/draft/2020-12/schema',
  $id: 'https://termy.dev/protocol/1/protocol.bundle.schema.json',
  title: 'ControlMessage',
  description:
    'GENERATED bundle of protocol/schema/. Do not edit; run `npm run bundle` in protocol/.',
  oneOf: messageNames.map((n) => ({ $ref: `#/$defs/${n}` })),
  $defs: defs
};

fs.mkdirSync(path.dirname(OUT), { recursive: true });
fs.writeFileSync(OUT, JSON.stringify(bundle, null, 2) + '\n');
console.log(
  `Wrote ${path.relative(ROOT, OUT)} (${messageNames.length} messages, ${Object.keys(defs).length} defs)`
);
