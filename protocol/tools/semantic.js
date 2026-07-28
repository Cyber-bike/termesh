'use strict';

/**
 * Cross-field rules that JSON Schema cannot express.
 *
 * Doc 8.4 / 10.4 state these as prose ("index must run 0..n-1", "sentSize is
 * authoritative", "code is null exactly when success is true"). They are part of
 * the contract, so all three ends must enforce them and the shared fixtures must
 * cover them. Keeping them here means the fixture suite tests the same rules the
 * runtime does.
 */

const PER_FILE_LIMIT = 67108864; // 64 MiB
const PER_TRANSFER_LIMIT = 268435456; // 256 MiB
const MAX_PATH_BYTES = 1024;
const MAX_CONTROL_FRAME_BYTES = 65536;

/** @returns {string[]} list of violations, empty when the message is valid */
function checkSemantics(message) {
  const errors = [];
  const payload = message && message.payload;
  if (!payload || typeof payload !== 'object') return errors;

  if (message.type === 'transfer.start') {
    const entries = Array.isArray(payload.entries) ? payload.entries : [];

    entries.forEach((entry, i) => {
      if (entry.index !== i) {
        errors.push(`entries[${i}].index is ${entry.index}, expected ${i} (index must run 0..n-1 with no gaps)`);
      }
      if (utf8Length(entry.relativePath) > MAX_PATH_BYTES) {
        errors.push(`entries[${i}].relativePath exceeds ${MAX_PATH_BYTES} UTF-8 bytes`);
      }
      if (entry.size > PER_FILE_LIMIT) {
        errors.push(`entries[${i}].size ${entry.size} exceeds the 64 MiB per-file limit`);
      }
    });

    const seen = new Set();
    for (const entry of entries) {
      const key = entry.relativePath;
      if (seen.has(key)) errors.push(`duplicate relativePath ${key}`);
      seen.add(key);
    }

    if (entries.length > 0 && payload.rootNote !== entries[0].relativePath) {
      errors.push('rootNote must equal entries[0].relativePath (doc 10.1: the root note is the first item)');
    }

    const total = entries.reduce((sum, entry) => sum + (entry.size || 0), 0);
    if (total > PER_TRANSFER_LIMIT) {
      errors.push(`total size ${total} exceeds the 256 MiB per-transfer limit`);
    }
  }

  if (message.type === 'transfer.result') {
    const codeIsNull = payload.code === null;
    if (codeIsNull !== payload.success) {
      errors.push('code must be null exactly when success is true');
    }
  }

  if (message.type === 'terminal.close') {
    if (payload.reason === 'shell_exited' && payload.exitCode === null) {
      errors.push('reason=shell_exited requires a non-null exitCode');
    }
  }

  const encoded = Buffer.byteLength(JSON.stringify(message), 'utf8');
  if (encoded > MAX_CONTROL_FRAME_BYTES) {
    errors.push(`encoded control frame is ${encoded} bytes, over the 65536 limit (doc 8.3)`);
  }

  return errors;
}

function utf8Length(value) {
  return typeof value === 'string' ? Buffer.byteLength(value, 'utf8') : 0;
}

module.exports = { checkSemantics, PER_FILE_LIMIT, PER_TRANSFER_LIMIT, MAX_PATH_BYTES, MAX_CONTROL_FRAME_BYTES };
