/**
 * v2.0 note-transfer sender (doc §8.4/8.6/10), driving the
 * `transferManifest`/`transferChunk`/`transferFileEnd`/`transferComplete`
 * frames (`terminalStreamFrame.ts`) over the same `ByteStream`/`openStream`
 * seam `TerminalStreamTransport` uses - see that module and
 * `remoteDirectoryTreeSource.ts` for why this seam is already safe to build
 * on ahead of the plugin's connection-management UI landing.
 *
 * Deliberately mirrors V1's `transferSender.ts` in shape (manifest,
 * credit-paced chunks, fileEnd, complete, same `CreditWindow`/`chunkify`)
 * since the send-side pacing logic doesn't change between the two wire
 * formats - only how each step is framed on the wire does.
 *
 * Unlike V1's `TransferSender`, `run()` never throws: every failure mode
 * (rejected manifest, mid-transfer error, protocol violation, local read
 * failure) resolves to a `TransferOutcome` with `success: false`, matching
 * the "one verdict, not an exception" contract `RemoteService.transfer`
 * (V1) already established for callers.
 */

import { CreditWindow, chunkify } from './creditWindow.ts';
import type { CollectedFile } from './noteCollector.ts';
import {
  encodeTerminalStreamFrame,
  TerminalStreamFrameDecoder,
  type TerminalStreamFrame,
} from './terminalStreamFrame.ts';
import type { ByteStream } from './terminalStreamTransport.ts';

export interface TransferOutcome {
  success: boolean;
  code: string | null;
  message: string;
}

export interface TransferSenderCallbacks {
  onProgress?(sentBytes: number, totalBytes: number): void;
}

/** `SESSION_LIMIT_REACHED: at most 8...` -> `SESSION_LIMIT_REACHED`. Mirrors `terminalStreamTransport.ts`'s local helper. */
function errorCode(message: string): string {
  const match = /^([A-Z][A-Z0-9_]*):/.exec(message);
  return match ? match[1] : 'PROTOCOL_ERROR';
}

async function readOneFrame(stream: ByteStream, decoder: TerminalStreamFrameDecoder): Promise<TerminalStreamFrame> {
  for (;;) {
    const frame = decoder.nextFrame();
    if (frame) return frame;
    const chunk = await stream.read();
    if (chunk === null) {
      throw new Error('PROTOCOL_ERROR: the agent closed the stream before responding');
    }
    decoder.push(chunk);
  }
}

export class TransferStreamSender {
  constructor(
    private readonly openStream: () => Promise<ByteStream>,
    private readonly transferId: string,
    private readonly files: CollectedFile[],
    private readonly readFile: (path: string) => Promise<Uint8Array>,
    /** Doc §7.6: lands the transfer in this session's cwd instead of the agent's configured receive root, when known. */
    private readonly sessionId: string | null = null,
    /** Directory-tree "drop onto this node" (candidate doc §4.1 point 4): wins outright over sessionId/receive_root when present. */
    private readonly targetPath: string | null = null,
    private readonly callbacks: TransferSenderCallbacks = {},
  ) {}

  async run(): Promise<TransferOutcome> {
    let stream: ByteStream;
    try {
      stream = await this.openStream();
    } catch (error) {
      return { success: false, code: 'TRANSFER_FAILED', message: describeError(error) };
    }

    try {
      const decoder = new TerminalStreamFrameDecoder();
      await stream.write(
        encodeTerminalStreamFrame({
          kind: 'transferManifest',
          payload: {
            transferId: this.transferId,
            rootNote: this.files[0].relativePath,
            entries: this.files.map((file) => ({
              index: file.index,
              relativePath: file.relativePath,
              size: file.size,
            })),
            sessionId: this.sessionId,
            targetPath: this.targetPath,
          },
        }),
      );

      const first = await readOneFrame(stream, decoder);
      if (first.kind === 'error') {
        return { success: false, code: errorCode(first.payload.message), message: first.payload.message };
      }
      if (first.kind !== 'transferAccepted') {
        return {
          success: false,
          code: 'PROTOCOL_ERROR',
          message: `expected transferAccepted, got ${first.kind}`,
        };
      }

      const window = new CreditWindow(first.payload.grantedBytes);
      const resultPromise = this.pumpUntilResult(stream, decoder, window);

      const total = this.files.reduce((sum, file) => sum + file.size, 0);
      let sent = 0;
      for (const file of this.files) {
        const bytes = await this.readFile(file.relativePath);
        for (const { offset, slice } of chunkify(bytes)) {
          await window.reserve(slice.length);
          await stream.write(
            encodeTerminalStreamFrame({
              kind: 'transferChunk',
              payload: { fileIndex: file.index, offset, data: slice },
            }),
          );
          sent += slice.length;
          this.callbacks.onProgress?.(sent, total);
        }
        // Sent even for an empty file: it is the only signal the agent gets that a zero-byte file exists (doc 10.4).
        await stream.write(
          encodeTerminalStreamFrame({
            kind: 'transferFileEnd',
            payload: { fileIndex: file.index, sentSize: bytes.length },
          }),
        );
      }
      await stream.write(encodeTerminalStreamFrame({ kind: 'transferComplete', payload: {} }));
      stream.finishWrite();

      return await resultPromise;
    } catch (error) {
      try {
        stream.finishWrite();
      } catch {
        // The stream is already gone; closing it was the goal anyway.
      }
      return { success: false, code: 'TRANSFER_FAILED', message: describeError(error) };
    }
  }

  /** Reads `transferCredit` grants and any final `transferResult`/`error`, never rejecting. */
  private async pumpUntilResult(
    stream: ByteStream,
    decoder: TerminalStreamFrameDecoder,
    window: CreditWindow,
  ): Promise<TransferOutcome> {
    for (;;) {
      let frame: TerminalStreamFrame;
      try {
        frame = await readOneFrame(stream, decoder);
      } catch (error) {
        const message = describeError(error);
        window.fail(new Error(message));
        return { success: false, code: 'PROTOCOL_ERROR', message };
      }

      if (frame.kind === 'transferCredit') {
        window.grant(frame.payload.grantedBytes);
        continue;
      }
      if (frame.kind === 'transferResult') {
        if (!frame.payload.success) window.fail(new Error(frame.payload.message || 'transfer failed'));
        return frame.payload;
      }
      if (frame.kind === 'error') {
        window.fail(new Error(frame.payload.message));
        return { success: false, code: errorCode(frame.payload.message), message: frame.payload.message };
      }
      // Anything else (e.g. a stray fs/terminal frame kind on a misbehaving
      // peer) is ignored rather than treated as fatal - keep waiting for a verdict.
    }
  }
}

function describeError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
