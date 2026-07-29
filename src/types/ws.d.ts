declare module 'ws' {
  import type { EventEmitter } from 'events';
  import type { IncomingMessage } from 'http';

  export type RawData = string | Buffer | ArrayBuffer | Buffer[];

  export class WebSocket extends EventEmitter {
    static readonly OPEN: number;
    readonly readyState: number;

    constructor(
      address: string,
      protocols?: string | string[],
      // `agent` is passed straight through to https.request; false means "do
      // not use the global agent", which is what keeps a stale pooled socket
      // from being reused for the upgrade.
      options?: { headers?: Record<string, string>; agent?: false },
    );

    close(code?: number, data?: string): void;
    send(data: string | Buffer | ArrayBuffer | ArrayBufferView, cb?: (error?: Error) => void): void;

    on(event: 'open', listener: () => void): this;
    on(event: 'message', listener: (data: RawData, isBinary: boolean) => void): this;
    on(event: 'close', listener: (code: number, reason: Buffer) => void): this;
    on(event: 'error', listener: (error: Error) => void): this;
  }

  export class WebSocketServer extends EventEmitter {
    constructor(options: { port: number });

    address(): { port: number } | string | null;
    close(cb?: () => void): void;

    on(
      event: 'connection',
      listener: (socket: WebSocket, request: IncomingMessage) => void,
    ): this;
    on(event: 'error', listener: (error: Error) => void): this;
    once(event: 'listening', listener: () => void): this;
    once(event: 'error', listener: (error: Error) => void): this;
  }

  export default WebSocket;
}
