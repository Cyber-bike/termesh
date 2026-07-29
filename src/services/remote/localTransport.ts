import type { PtyClient } from '@/services/server/ptyClient';
import type { PtyConfig, ShellEvent } from '@/services/server/types';
import {
  toDisposable,
  type Disposable,
  type TerminalExitEvent,
  type TerminalOpenOptions,
  type TerminalSessionInfo,
  type TerminalTransport,
} from './transport';

export class LocalTerminalTransport implements TerminalTransport {
  private readonly ptyClient: PtyClient;
  private readonly config: PtyConfig;
  private sessionId: string | null = null;

  constructor(
    ptyClient: PtyClient,
    config: PtyConfig,
  ) {
    this.ptyClient = ptyClient;
    this.config = config;
  }

  async open(options: TerminalOpenOptions): Promise<TerminalSessionInfo> {
    if (this.sessionId) {
      throw new Error('Terminal transport is already open');
    }

    const sessionId = await this.ptyClient.init({
      ...this.config,
      cwd: options.cwd ?? this.config.cwd,
      cols: options.cols,
      rows: options.rows,
    });
    this.sessionId = sessionId;
    return {
      sessionId,
      shell: this.config.shell_type ?? 'default',
    };
  }

  write(data: Uint8Array): void {
    this.ptyClient.writeBinary(this.requireSessionId(), data);
  }

  resize(cols: number, rows: number): void {
    this.ptyClient.resize(this.requireSessionId(), cols, rows);
  }

  close(): Promise<void> {
    const sessionId = this.sessionId;
    if (!sessionId) return Promise.resolve();
    this.sessionId = null;
    this.ptyClient.destroySession(sessionId);
    return Promise.resolve();
  }

  onData(handler: (data: Uint8Array) => void): Disposable {
    return toDisposable(this.ptyClient.onSessionOutput(this.requireSessionId(), handler));
  }

  onExit(handler: (event: TerminalExitEvent) => void): Disposable {
    return toDisposable(this.ptyClient.onSessionExit(this.requireSessionId(), (exitCode) => {
      handler({ exitCode, reason: 'shell_exited' });
    }));
  }

  onError(handler: (code: string, message: string) => void): Disposable {
    return toDisposable(this.ptyClient.onSessionError(this.requireSessionId(), handler));
  }

  onShellEvent(handler: (event: ShellEvent) => void): Disposable {
    return toDisposable(this.ptyClient.onSessionShellEvent(this.requireSessionId(), handler));
  }

  private requireSessionId(): string {
    if (!this.sessionId) {
      throw new Error('Terminal transport is not open');
    }
    return this.sessionId;
  }
}