'use strict';

/**
 * Compiles every schema and runs the shared fixture suite.
 *
 * Fixture naming: fixtures/valid/<type>.<case>.json must pass both JSON Schema
 * and the semantic rules; fixtures/invalid/<type>.<case>.json must fail at least
 * one of them. The Rust side runs the same directory, so a fixture added here is
 * automatically a cross-end contract test (doc 8.1).
 */

const fs = require('fs');
const path = require('path');
const Ajv = require('ajv/dist/2020');
const addFormats = require('ajv-formats');
const { checkSemantics } = require('./semantic');

const SCHEMA_DIR = path.join(__dirname, '..', 'schema');
const MESSAGE_DIR = path.join(SCHEMA_DIR, 'messages');
const FIXTURE_DIR = path.join(__dirname, '..', 'fixtures');

const readJson = (file) => JSON.parse(fs.readFileSync(file, 'utf8'));

const ajv = new Ajv({ strict: true, allErrors: true, allowUnionTypes: true });
addFormats(ajv);

ajv.addSchema(readJson(path.join(SCHEMA_DIR, 'common.schema.json')));
ajv.addSchema(readJson(path.join(SCHEMA_DIR, 'control-envelope.schema.json')));

const messageFiles = fs.readdirSync(MESSAGE_DIR).filter((f) => f.endsWith('.schema.json')).sort();
const validators = new Map();

let failures = 0;
const fail = (msg) => {
  console.error(`  FAIL ${msg}`);
  failures += 1;
};

console.log(`Compiling ${messageFiles.length} message schemas`);
for (const file of messageFiles) {
  const schema = readJson(path.join(MESSAGE_DIR, file));
  const type = schema.properties && schema.properties.type && schema.properties.type.const;
  if (!type) {
    fail(`${file}: schema does not pin properties.type.const`);
    continue;
  }
  try {
    validators.set(type, ajv.compile(schema));
  } catch (err) {
    fail(`${file}: ${err.message}`);
  }
}

// Every type listed in the envelope enum must have a schema, and vice versa.
const envelope = readJson(path.join(SCHEMA_DIR, 'control-envelope.schema.json'));
const declared = new Set(envelope.properties.type.enum);
for (const type of declared) {
  if (!validators.has(type)) fail(`envelope declares "${type}" but messages/ has no schema for it`);
}
for (const type of validators.keys()) {
  if (!declared.has(type)) fail(`messages/ defines "${type}" but the envelope enum omits it`);
}

function checkFixture(file, expectValid) {
  const message = readJson(file);
  const name = path.basename(file);
  const validate = validators.get(message.type);

  if (!validate) {
    // An unknown type is itself a protocol error, so it is a legitimate invalid fixture.
    if (expectValid) fail(`${name}: no schema for type "${message.type}"`);
    return;
  }

  const schemaOk = validate(message);
  const semanticErrors = checkSemantics(message);
  const ok = schemaOk && semanticErrors.length === 0;

  if (ok !== expectValid) {
    if (expectValid) {
      const detail = schemaOk ? semanticErrors.join('; ') : ajv.errorsText(validate.errors, { separator: '; ' });
      fail(`${name}: expected valid, got: ${detail}`);
    } else {
      fail(`${name}: expected invalid, but it passed every check`);
    }
  }
}

for (const [dir, expectValid] of [['valid', true], ['invalid', false]]) {
  const full = path.join(FIXTURE_DIR, dir);
  const files = fs.existsSync(full) ? fs.readdirSync(full).filter((f) => f.endsWith('.json')).sort() : [];
  console.log(`Checking ${files.length} ${dir} fixtures`);
  for (const file of files) checkFixture(path.join(full, file), expectValid);
  if (expectValid && files.length === 0) fail('no valid fixtures found');
}

// Each message type needs at least one positive fixture, otherwise a schema can
// rot unnoticed.
const covered = new Set(
  fs.readdirSync(path.join(FIXTURE_DIR, 'valid'))
    .filter((f) => f.endsWith('.json'))
    .map((f) => readJson(path.join(FIXTURE_DIR, 'valid', f)).type)
);
for (const type of validators.keys()) {
  if (!covered.has(type)) fail(`no valid fixture covers "${type}"`);
}

if (failures > 0) {
  console.error(`\n${failures} failure(s)`);
  process.exit(1);
}
console.log('\nAll schema and fixture checks passed');
