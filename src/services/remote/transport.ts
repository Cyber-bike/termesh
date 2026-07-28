/**
 * Terminal transport abstraction (doc 5.2).
 *
 * Four event channels, not two: `TerminalInstance` consumes onSessionOutput,
 * onSessionExit, onSessionError and onSessionShellEvent from the existing
 * PtyClient, so a narrower interface would strand shell integration when the
 * terminal is remote.
 *
 * One instance per session. PtyClient's methods all take a sessionId because it
 * multiplexes; an implementation of this interface holds that id internally.
 */

export interface TerminalOpenOptions {
  cols: number;
  rows: number;
  cwd?: string;
}

export interface TerminalSessionInfo {
  sessionId: string;
  shell: string;
}

export interface TerminalExitEvent {
  exitCode: number | null;
  reason: 'user' | 'peer_disconnected' | 'shell_exited' | 'error';
}

/** Shape-identical to the local ShellEvent so it can be forwarded unchanged. */
export interface ShellEventPayload {
  type: 'prompt_start' | 'command_start' | 'command_executed' | 'command_end';
  source: 'osc133' | 'osc633';
  exitCode: number | null;
}

export interface Disposable {
  dispose(): void;
}

export interface TerminalTransport {
  open(options: TerminalOpenOptions): Promise<TerminalSessionInfo>;
  write(data: Uint8Array): void;
  resize(cols: number, rows: number): void;
  close(reason?: string): Promise<void>;

  onData(handler: (data: Uint8Array) => void): Disposable;
  onExit(handler: (event: TerminalExitEvent) => void): Disposable;
  onError(handler: (code: string, message: string) => void): Disposable;
  onShellEvent(handler: (event: ShellEventPayload) => void): Disposable;
}

/** Turns an unsubscribe callback into the Disposable the interface expects. */
export function toDisposable(unsubscribe: () => void): Disposable {
  let disposed = false;
  return {
    dispose(): void {
      if (disposed) return;
      disposed = true;
      unsubscribe();
    },
  };
}

/** Collects disposables so a transport can be torn down in one call. */
export class DisposableBag implements Disposable {
  private items: Disposable[] = [];

  add(item: Disposable): Disposable {
    this.items.push(item);
    return item;
  }

  dispose(): void {
    const items = this.items;
    this.items = [];
    for (const item of items) {
      try {
        item.dispose();
      } catch {
        // A failing listener must not prevent the rest from being released.
      }
    }
  }
}
