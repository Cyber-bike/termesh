'use strict';

/**
 * Checks openapi.yaml against the rules the doc fixes in chapter 6, not just
 * against the OpenAPI meta-schema. The interesting failures here are "someone
 * used a status code the doc forbids" or "a 429 forgot Retry-After", which a
 * generic validator would happily accept.
 */

const fs = require('fs');
const path = require('path');
const YAML = require('yaml');
const Ajv = require('ajv/dist/2020');
const addFormats = require('ajv-formats');

const ROOT = path.join(__dirname, '..');
const COMMON_ID = 'https://termy.dev/protocol/1/common.schema.json';

// Doc 6.2: success is only 200/201/204, failures only these.
const ALLOWED_STATUS = new Set(['200', '201', '204', '400', '401', '403', '404', '409', '429', '500']);

const EXPECTED_OPERATIONS = [
  ['/v1/auth/login', 'post'],
  ['/v1/auth/register', 'post'],
  ['/v1/devices/pairing-codes', 'post'],
  ['/v1/devices/pairing-codes/{id}', 'delete'],
  ['/v1/devices/register', 'post'],
  ['/v1/devices', 'get'],
  ['/v1/devices/{id}', 'delete']
];

let failures = 0;
const fail = (msg) => {
  console.error(`  FAIL ${msg}`);
  failures += 1;
};

const doc = YAML.parse(fs.readFileSync(path.join(ROOT, 'openapi.yaml'), 'utf8'));

if (!/^3\.1\.\d+$/.test(doc.openapi || '')) fail(`openapi must be 3.1.x, got "${doc.openapi}"`);

// --- operations -------------------------------------------------------------

const actual = [];
for (const [p, item] of Object.entries(doc.paths || {})) {
  for (const method of Object.keys(item)) {
    if (['get', 'post', 'put', 'patch', 'delete'].includes(method)) actual.push([p, method]);
  }
}

for (const [p, method] of EXPECTED_OPERATIONS) {
  if (!actual.some(([ap, am]) => ap === p && am === method)) fail(`missing operation ${method.toUpperCase()} ${p}`);
}
for (const [p, method] of actual) {
  if (!EXPECTED_OPERATIONS.some(([ep, em]) => ep === p && em === method)) {
    fail(`undocumented operation ${method.toUpperCase()} ${p} (add it to the explicit contract list)`);
  }
}

const operationIds = new Set();
for (const [p, item] of Object.entries(doc.paths || {})) {
  for (const [method, op] of Object.entries(item)) {
    if (!['get', 'post', 'put', 'patch', 'delete'].includes(method)) continue;
    const where = `${method.toUpperCase()} ${p}`;

    if (!op.operationId) fail(`${where}: missing operationId`);
    else if (operationIds.has(op.operationId)) fail(`${where}: duplicate operationId ${op.operationId}`);
    else operationIds.add(op.operationId);

    for (const [status, response] of Object.entries(op.responses || {})) {
      if (!ALLOWED_STATUS.has(status)) {
        fail(`${where}: status ${status} is not in the doc 6.2 allowlist`);
      }

      const resolved = resolveRef(response);

      if (status === '429') {
        const header = resolved.headers && resolved.headers['Retry-After'];
        if (!header) fail(`${where}: 429 must declare a Retry-After header (doc 6.2)`);
        else if (header.required !== true) fail(`${where}: Retry-After must be required`);
      }

      const isError = Number(status) >= 400;
      if (isError) {
        const schema = resolved.content && resolved.content['application/json'] &&
          resolved.content['application/json'].schema;
        if (!schema) fail(`${where}: ${status} must return a JSON body`);
        else if (schema.$ref !== '#/components/schemas/Error') {
          fail(`${where}: ${status} must use the shared Error schema, got ${JSON.stringify(schema)}`);
        }
      }

      if (status === '204' && resolved.content) fail(`${where}: 204 must not declare a body`);
    }
  }
}

function resolveRef(node) {
  if (!node || !node.$ref) return node || {};
  const parts = node.$ref.replace(/^#\//, '').split('/');
  let cur = doc;
  for (const part of parts) cur = cur && cur[part];
  if (!cur) fail(`unresolvable $ref ${node.$ref}`);
  return cur || {};
}

// --- schemas ----------------------------------------------------------------

const ajv = new Ajv({ strict: false, allErrors: true });
addFormats(ajv);
ajv.addSchema(JSON.parse(fs.readFileSync(path.join(ROOT, 'schema', 'common.schema.json'), 'utf8')));

// Rewrite the file-relative refs the OpenAPI document uses into the absolute $id
// ajv knows about, then compile every component schema for real.
const rewrite = (node) => {
  if (Array.isArray(node)) return node.map(rewrite);
  if (node && typeof node === 'object') {
    const out = {};
    for (const [k, v] of Object.entries(node)) {
      if (k === '$ref' && typeof v === 'string' && v.startsWith('./schema/common.schema.json#')) {
        out[k] = v.replace('./schema/common.schema.json#', `${COMMON_ID}#`);
      } else if (k === '$ref' && typeof v === 'string' && v.startsWith('#/components/schemas/')) {
        out[k] = `#/$defs/${v.slice('#/components/schemas/'.length)}`;
      } else {
        out[k] = rewrite(v);
      }
    }
    return out;
  }
  return node;
};

const componentSchemas = rewrite(doc.components.schemas);
const bundle = { $id: 'https://termy.dev/protocol/1/openapi-components.json', $defs: componentSchemas };

try {
  ajv.addSchema(bundle);
  for (const name of Object.keys(componentSchemas)) {
    ajv.compile({ $ref: `${bundle.$id}#/$defs/${name}` });
  }
  console.log(`Compiled ${Object.keys(componentSchemas).length} component schemas`);
} catch (err) {
  fail(`component schema compilation: ${err.message}`);
}

// Doc 8.1: every object is closed.
const walkObjects = (node, trail) => {
  if (Array.isArray(node)) return node.forEach((n, i) => walkObjects(n, `${trail}[${i}]`));
  if (!node || typeof node !== 'object') return;
  if (node.type === 'object' && node.additionalProperties !== false) {
    fail(`${trail}: object schema must set additionalProperties: false`);
  }
  for (const [k, v] of Object.entries(node)) walkObjects(v, `${trail}/${k}`);
};
walkObjects(doc.components.schemas, 'components/schemas');

// --- consistency with the WSS side -----------------------------------------

const common = JSON.parse(fs.readFileSync(path.join(ROOT, 'schema', 'common.schema.json'), 'utf8'));
// Doc 6.2 says 900. That is now the relay's default rather than a fixed value,
// because a client holding no credentials cannot survive a 15 minute token, so
// what the contract has to pin is the range a client must be prepared to see -
// and above all that it stays an integer count of seconds.
const loginExpires = doc.components.schemas.LoginResponse.properties.expiresIn;
if (loginExpires.const !== undefined) {
  fail('LoginResponse.expiresIn must not be a const: the lifetime is deployment-configurable');
}
if (loginExpires.type !== 'integer') fail(`LoginResponse.expiresIn should be an integer, got ${loginExpires.type}`);
if (loginExpires.minimum !== 60 || loginExpires.maximum !== 86400) {
  fail(
    'LoginResponse.expiresIn should allow 60..86400 to match the relay\'s ' +
      `TERMY_ACCESS_TOKEN_TTL_SECS bounds, got ${loginExpires.minimum}..${loginExpires.maximum}`
  );
}

const maxDevices = doc.components.schemas.DeviceList.properties.devices.maxItems;
if (maxDevices !== 32) fail(`DeviceList.devices.maxItems should match the 32-device quota, got ${maxDevices}`);

const platformRef = doc.components.schemas.Device.properties.platform.$ref;
if (!platformRef.endsWith('#/$defs/Platform')) fail('Device.platform must reuse the shared Platform enum');
if (!common.$defs.Platform.enum.includes('windows-x64')) fail('common Platform enum drifted');

if (failures > 0) {
  console.error(`\n${failures} failure(s)`);
  process.exit(1);
}
console.log('OpenAPI document passes structural and doc-conformance checks');
