/**
 * Connection-code pre-validation (v2.0 doc 5.2, plan §3.2 "连接码解析").
 *
 * A connection code is the string form of an iroh `EndpointTicket`
 * (`@number0/iroh` 1.1.0 naming - the implementation doc's `NodeTicket` is
 * the same thing under iroh's pre-1.0 name): the literal prefix `endpoint`
 * followed by unpadded lowercase RFC 4648 base32. Verified against a real
 * ticket emitted by the actual library, not inferred from docs.
 *
 * This module is the cheap, dependency-free half of parsing: instant
 * feedback for the "添加设备" paste box (doc 6.4's `TICKET_INVALID` before a
 * connection is ever attempted). It cannot derive the `EndpointId`, so the
 * authoritative parse - `EndpointTicket.fromString` -> `EndpointAddr` -
 * stays behind the `ConnectionCodeParser` seam and is supplied by the iroh
 * binding once the A0 spike settles how that binding is loaded. Anything
 * accepted here can still be rejected there; nothing rejected here would
 * have survived the real parser.
 */

/** Ticket variant prefix as emitted by iroh's serde for `EndpointTicket`. */
export const CONNECTION_CODE_PREFIX = 'endpoint';

/** Unpadded RFC 4648 base32, lowercase (iroh's ticket body alphabet). */
const BASE32_BODY = /^[a-z2-7]+$/;

/**
 * The base32 length of a bare 32-byte `EndpointId` (ceil(32*8/5) = 52). A
 * real ticket also carries address information, so its body is strictly
 * longer - anything shorter cannot even name a device.
 */
const MIN_BODY_LENGTH = 52;

export type ConnectionCodeProblem =
  | 'empty'
  | 'wrong-prefix'
  | 'bad-characters'
  | 'too-short';

export type ConnectionCodeCheck =
  | { ok: true; normalized: string }
  /** `code` is the doc §13 error code every pre-validation failure maps to. */
  | { ok: false; code: 'TICKET_INVALID'; problem: ConnectionCodeProblem };

/**
 * Whitespace is stripped everywhere, not just trimmed: codes get copied out
 * of terminals, where line wrapping and tmux-style copy modes insert
 * newlines mid-string. Base32 has no whitespace, so removal is lossless.
 * Case is folded to lowercase because base32 decoding is case-insensitive
 * and iroh's own output is lowercase.
 */
export function normalizeConnectionCode(raw: string): string {
  return raw.replace(/\s+/g, '').toLowerCase();
}

export function checkConnectionCode(raw: string): ConnectionCodeCheck {
  const normalized = normalizeConnectionCode(raw);
  const invalid = (problem: ConnectionCodeProblem): ConnectionCodeCheck => ({
    ok: false,
    code: 'TICKET_INVALID',
    problem,
  });

  if (normalized === '') return invalid('empty');
  if (!normalized.startsWith(CONNECTION_CODE_PREFIX)) return invalid('wrong-prefix');

  const body = normalized.slice(CONNECTION_CODE_PREFIX.length);
  if (body.length > 0 && !BASE32_BODY.test(body)) return invalid('bad-characters');
  if (body.length < MIN_BODY_LENGTH) return invalid('too-short');

  return { ok: true, normalized };
}

/** What the authoritative parser must recover from a normalized code. */
export interface ParsedConnectionCode {
  /** Base32 string form of the `EndpointId` - `PairedDeviceStore`'s dedup key. */
  nodeId: string;
}

/**
 * The seam for the real `EndpointTicket.fromString` parse, implemented by
 * the iroh binding integration (A0). Throws on codes the pre-validation
 * could not catch.
 */
export type ConnectionCodeParser = (normalizedCode: string) => ParsedConnectionCode;
