'use strict';

/**
 * Generates generated/typescript/ from the schemas. Output is checked in and
 * must never be hand-edited (doc 8.1); CI regenerates and fails on a diff.
 */

const fs = require('fs');
const path = require('path');
const { compile } = require('json-schema-to-typescript');

const ROOT = path.join(__dirname, '..');
const MESSAGE_DIR = path.join(ROOT, 'schema', 'messages');
const OUT_DIR = path.join(ROOT, 'generated', 'typescript');

const BANNER = `/**
 * GENERATED FILE - DO NOT EDIT.
 * Source: protocol/schema/. Regenerate with \`npm run generate\` in protocol/.
 */`;

const options = {
  bannerComment: '',
  additionalProperties: false,
  declareExternallyReferenced: true,
  enableConstEnums: false,
  cwd: path.join(ROOT, 'schema'),
  style: { singleQuote: true, printWidth: 100 }
};

const pascal = (s) =>
  s.replace(/[.\-_](\w)/g, (_, c) => c.toUpperCase()).replace(/^(\w)/, (_, c) => c.toUpperCase());

async function main() {
  fs.mkdirSync(OUT_DIR, { recursive: true });

  const files = fs.readdirSync(MESSAGE_DIR).filter((f) => f.endsWith('.schema.json')).sort();
  const parts = [BANNER, ''];
  const unionMembers = [];
  const typeMap = [];

  for (const file of files) {
    const schema = JSON.parse(fs.readFileSync(path.join(MESSAGE_DIR, file), 'utf8'));
    const wireType = schema.properties.type.const;
    const typeName = pascal(wireType) + 'Message';

    // Inline the envelope so each message is a standalone, closed interface.
    const flattened = await flatten(schema);
    const ts = await compile({ ...flattened, title: typeName }, typeName, options);

    parts.push(ts.trim(), '');
    unionMembers.push(typeName);
    typeMap.push([wireType, typeName]);
  }

  parts.push(`export type ControlMessage =\n${unionMembers.map((t) => `    | ${t}`).join('\n')};`, '');
  parts.push(
    'export interface ControlMessageByType {',
    ...typeMap.map(([wire, ts]) => `    '${wire}': ${ts};`),
    '}',
    ''
  );
  parts.push(
    'export const CONTROL_MESSAGE_TYPES = [',
    ...typeMap.map(([wire]) => `    '${wire}',`),
    "] as const;",
    '',
    'export type ControlMessageType = (typeof CONTROL_MESSAGE_TYPES)[number];',
    ''
  );

  const outFile = path.join(OUT_DIR, 'messages.ts');
  fs.writeFileSync(outFile, parts.join('\n'));
  console.log(`Wrote ${path.relative(ROOT, outFile)} (${files.length} messages)`);
}

/**
 * json-schema-to-typescript does not follow a sibling $ref into a base object,
 * so merge the envelope's properties in before compiling. The message's own
 * properties win, which is what pins type/requestId/sessionId per message.
 */
async function flatten(schema) {
  const envelope = JSON.parse(
    fs.readFileSync(path.join(ROOT, 'schema', 'control-envelope.schema.json'), 'utf8')
  );
  const { $ref, unevaluatedProperties, ...rest } = schema;
  const merged = {
    ...rest,
    type: 'object',
    properties: { ...envelope.properties, ...schema.properties },
    required: envelope.required,
    additionalProperties: false
  };
  return absolutiseRefs(merged);
}

/**
 * The envelope refs common.schema.json relative to schema/, the messages relative
 * to schema/messages/. A single resolver cwd cannot serve both, so rewrite every
 * common.schema.json ref to an absolute path before compiling.
 */
const COMMON_PATH = path.join(ROOT, 'schema', 'common.schema.json');

function absolutiseRefs(node) {
  if (Array.isArray(node)) return node.map(absolutiseRefs);
  if (node && typeof node === 'object') {
    const out = {};
    for (const [k, v] of Object.entries(node)) {
      if (k === '$ref' && typeof v === 'string' && v.includes('common.schema.json#')) {
        out[k] = COMMON_PATH + v.slice(v.indexOf('#'));
      } else {
        out[k] = absolutiseRefs(v);
      }
    }
    return out;
  }
  return node;
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
