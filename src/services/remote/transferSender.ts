/**
 * Drives one note transfer over the control connection (doc 8.4, 8.6, 10).
 *
 * Independent of Obsidian: it takes a `ControlChannel` and a byte-reader, so the
 * whole send sequence - manifest, credit-paced chunks, fileEnd, complete - is
 * exercisable without a vault or a socket.
 */

import { CreditWindow, chunkify } from './creditWindow.ts';
import { encodeFrame, KIND_FILE_CHUNK } from './frameCodec.ts';
import type { CollectedFile } from './noteCollector.ts';

export interface ControlChannel {
  sendJson(message: unknown): void;
  sendBinary(frame: Uint8Array): void;
}

export interface TransferCallbacks {
  /** Bytes sent so far, for a progress indicator. */
  onProgress?(sentBytes: number, totalBytes: number): void;
}

export interface TransferOutcome {
  success: boolean;
  code: string | null;
  message: string;
}

export class TransferSender {
  private readonly channel: ControlChannel;
  private readonly deviceId: string;
  readonly transferId: string;
  private readonly files: CollectedFile[];
  private readonly readFile: (path: string) => Promise<Uint8Array>;
  private readonly callbacks: TransferCallbacks;
  private readonly window: CreditWindow;
  private settle: ((outcome: TransferOutcome) => void) | null = null;
  private finished = false;

  constructor(
    channel: ControlChannel,
    deviceId: string,
    transferId: string,
    files: CollectedFile[],
    readFile: (path: string) => Promise<Uint8Array>,
    initialCredit: number,
    callbacks: TransferCallbacks = {}
  ) {
    this.channel = channel;
    this.deviceId = deviceId;
    this.transferId = transferId;
    this.files = files;
    this.readFile = readFile;
    this.callbacks = callbacks;
    this.window = new CreditWindow(initialCredit);
  }

  /** Applies a `transfer.credit` grant. */
  grantCredit(grantedBytes: number): void {
    this.window.grant(grantedBytes);
  }

  /** Applies a `transfer.result`, ending the wait started by `run`. */
  complete(outcome: TransferOutcome): void {
    if (this.finished) return;
    this.finished = true;
    if (!outcome.success) {
      this.window.fail(new Error(outcome.message || 'transfer failed'));
    }
    this.settle?.(outcome);
  }

  /** Ends the transfer locally, e.g. the connection dropped. */
  abandon(message: string): void {
    try {
      this.channel.sendJson(
        envelope('transfer.abort', this.deviceId, {
          transferId: this.transferId,
          code: 'TRANSFER_FAILED',
        })
      );
    } finally {
      this.complete({ success: false, code: 'TRANSFER_FAILED', message });
    }
  }

  /**
   * Sends every file, then waits for the agent's verdict.
   *
   * Chunks are paced by the credit window, so a large attachment cannot fill the
   * relay's queue and stall the terminal (doc 8.6).
   */
  async run(): Promise<TransferOutcome> {
    const total = this.files.reduce((sum, file) => sum + file.size, 0);
    let sent = 0;

    const verdict = new Promise<TransferOutcome>((resolve) => {
      this.settle = resolve;
    });

    try {
      for (const file of this.files) {
        const bytes = await this.readFile(file.relativePath);

        for (const { offset, slice } of chunkify(bytes)) {
          await this.window.reserve(slice.length);
          this.channel.sendBinary(
            encodeFrame({
              kind: KIND_FILE_CHUNK,
              streamId: this.transferId,
              fileIndex: file.index,
              offset,
              payload: slice,
            })
          );
          sent += slice.length;
          this.callbacks.onProgress?.(sent, total);
        }

        // Sent even for an empty file: it is the only signal the agent gets that
        // a zero-byte file exists (doc 10.4).
        this.channel.sendJson(
          envelope('transfer.fileEnd', this.deviceId, {
            transferId: this.transferId,
            fileIndex: file.index,
            sentSize: bytes.length,
          })
        );
      }

      this.channel.sendJson(
        envelope('transfer.complete', this.deviceId, { transferId: this.transferId })
      );
    } catch (error) {
      if (!this.finished) {
        const message = error instanceof Error ? error.message : String(error);
        this.abandon(message);
      }
    }

    return verdict;
  }
}

export function envelope(
  type: string,
  deviceId: string,
  payload: unknown,
  extra: { requestId?: string | null; sessionId?: string | null } = {}
): Record<string, unknown> {
  return {
    protocolVersion: 1,
    type,
    requestId: extra.requestId ?? null,
    deviceId,
    sessionId: extra.sessionId ?? null,
    payload,
  };
}
