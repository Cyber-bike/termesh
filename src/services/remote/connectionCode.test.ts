import * as assert from 'node:assert/strict';
import test from 'node:test';

import { checkConnectionCode, normalizeConnectionCode } from './connectionCode.ts';

/**
 * A real ticket emitted by `@number0/iroh` 1.1.0's `EndpointTicket.toString()`
 * during the A0 probe (2026-07-31), not a synthetic string.
 */
const REAL_TICKET =
  'endpointactqqssc2bihkta43whj3xhgxbf6nq75f7wxi6zdldbgn65cjllpeaibab7qaaabxhjag';

test('a real ticket from the actual library passes', () => {
  const result = checkConnectionCode(REAL_TICKET);
  assert.deepEqual(result, { ok: true, normalized: REAL_TICKET });
});

test('uppercase and embedded whitespace are normalized away', () => {
  const mangled = `  ${REAL_TICKET.slice(0, 30).toUpperCase()}\n\t${REAL_TICKET.slice(30)}  `;
  const result = checkConnectionCode(mangled);
  assert.equal(result.ok, true);
  if (result.ok) assert.equal(result.normalized, REAL_TICKET);
});

test('normalizeConnectionCode leaves an already-clean code untouched', () => {
  assert.equal(normalizeConnectionCode(REAL_TICKET), REAL_TICKET);
});

test('an empty or whitespace-only paste is reported as empty', () => {
  for (const raw of ['', '   ', '\n\t']) {
    const result = checkConnectionCode(raw);
    assert.deepEqual(result, { ok: false, code: 'TICKET_INVALID', problem: 'empty' });
  }
});

test('a ticket with another variant prefix is rejected as wrong-prefix', () => {
  // iroh's pre-1.0 tickets began with "node"; a stale code from an old
  // agent build must produce a clear failure, not a confusing parse error.
  const result = checkConnectionCode(`node${REAL_TICKET.slice('endpoint'.length)}`);
  assert.deepEqual(result, { ok: false, code: 'TICKET_INVALID', problem: 'wrong-prefix' });
});

test('the prefix must be at the start, not merely contained', () => {
  const result = checkConnectionCode(`xx${REAL_TICKET}`);
  assert.deepEqual(result, { ok: false, code: 'TICKET_INVALID', problem: 'wrong-prefix' });
});

test('characters outside the base32 alphabet are rejected', () => {
  // 0, 1, 8, 9 are not in RFC 4648 base32; neither is punctuation.
  for (const bad of ['0', '1', '8', '9', '!', '=']) {
    const result = checkConnectionCode(`${REAL_TICKET.slice(0, 40)}${bad}${REAL_TICKET.slice(41)}`);
    assert.deepEqual(
      result,
      { ok: false, code: 'TICKET_INVALID', problem: 'bad-characters' },
      `"${bad}" must be rejected`
    );
  }
});

test('a body too short to even hold an EndpointId is rejected', () => {
  const result = checkConnectionCode('endpointabc234');
  assert.deepEqual(result, { ok: false, code: 'TICKET_INVALID', problem: 'too-short' });
});

test('the bare prefix with no body at all is rejected', () => {
  const result = checkConnectionCode('endpoint');
  assert.deepEqual(result, { ok: false, code: 'TICKET_INVALID', problem: 'too-short' });
});
