/**
 * Sender-side credit window (doc 8.6).
 *
 * The plugin may never have more than `granted` cumulative bytes in flight. This
 * is what stops a large attachment from filling the relay's queue and stalling
 * the terminal, so it is enforced here rather than trusted to arrive slowly
 * enough on its own.
 */

export const FILE_CHUNK_BYTES = 256 * 1024;

export class CreditWindow {
  private granted: number;
  private sent = 0;
  private waiters: Array<() => void> = [];
  private failure: Error | null = null;

  constructor(initialGranted: number) {
    this.granted = initialGranted;
  }

  get availableBytes(): number {
    return Math.max(0, this.granted - this.sent);
  }

  get sentBytes(): number {
    return this.sent;
  }

  get grantedBytes(): number {
    return this.granted;
  }

  /**
   * Raises the ceiling. `grantedBytes` is cumulative and monotonic, so a message
   * that arrives out of order or is replayed cannot shrink the window.
   */
  grant(grantedBytes: number): void {
    if (grantedBytes > this.granted) {
      this.granted = grantedBytes;
      this.release();
    }
  }

  /** Blocks until `bytes` fit inside the window. */
  async reserve(bytes: number): Promise<void> {
    if (this.failure) throw this.failure;

    while (this.sent + bytes > this.granted) {
      await new Promise<void>((resolve) => {
        this.waiters.push(resolve);
      });
      if (this.failure) throw this.failure;
    }

    this.sent += bytes;
  }

  /** Wakes every waiter so an aborted transfer cannot leave a sender hanging. */
  fail(error: Error): void {
    this.failure = error;
    this.release();
  }

  private release(): void {
    const waiting = this.waiters;
    this.waiters = [];
    for (const resolve of waiting) resolve();
  }
}

/** Splits a payload into protocol-sized chunks. */
export function* chunkify(
  data: Uint8Array,
  chunkBytes = FILE_CHUNK_BYTES
): Generator<{ offset: number; slice: Uint8Array }> {
  if (data.length === 0) return;
  for (let offset = 0; offset < data.length; offset += chunkBytes) {
    yield { offset, slice: data.subarray(offset, Math.min(offset + chunkBytes, data.length)) };
  }
}
