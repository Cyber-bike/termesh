/**
 * Per-device connection lifecycle (v2.0 doc 6.3/6.4): the control end's
 * counterpart to the agent's `serve.rs`.
 *
 * Owns the one iroh Endpoint (lazily bound on first use, with the
 * controller's persisted identity - plan §3.2), one `Connection` per paired
 * device, and the wiring into `PairedDeviceStore` (`markConnected` /
 * `setOnline`, doc 5.2). Terminal sessions ride these connections as
 * bi-streams: `createTerminalTransport` hands out a
 * `TerminalStreamTransport` bound to the device's connection.
 *
 * The binding itself arrives through the injected `loadIroh` - in the real
 * plugin that resolves `@number0/iroh` from the plugin directory's
 * node_modules (A0 step-5 layout); in tests it returns a fake. Nothing
 * here imports the package at build time.
 */

import type { PairedDeviceStore } from './pairedDeviceStore.ts';
import type { Disposable } from './transport.ts';
import { toDisposable } from './transport.ts';
import { TerminalStreamTransport } from './terminalStreamTransport.ts';
import {
  ALPN_TERMINAL,
  terminalStreamFactory,
  type IrohConnection,
  type IrohEndpoint,
  type IrohModule,
} from './irohStreams.ts';

export type DeviceConnectionStatus =
  | { state: 'disconnected' }
  | { state: 'connecting' }
  | { state: 'connected' }
  | { state: 'error'; code: string; message: string };

export interface DeviceConnectionsDependencies {
  loadIroh: () => Promise<IrohModule>;
  store: PairedDeviceStore;
  /**
   * Persisted controller identity seed (32 bytes), or null on first run.
   * When null a fresh identity is generated and reported through
   * `onIdentityCreated` so the caller can persist it (plan §3.2:
   * 控制端 Endpoint 持久化身份).
   */
  identitySeed: number[] | null;
  onIdentityCreated: (seed: number[]) => void;
  /** 'production' = n0 relays + discovery; 'loopback' = local dev/tests. */
  profile?: 'production' | 'loopback';
  now?: () => string;
}

interface ActiveConnection {
  connection: IrohConnection;
  /** Bumped on every disconnect so a stale `closed()` watcher of a previous
   * connection cannot clobber the state of a newer one. */
  generation: number;
}

export class DeviceConnectionManager {
  private readonly deps: Required<Pick<DeviceConnectionsDependencies, 'profile' | 'now'>> &
    DeviceConnectionsDependencies;
  private module: IrohModule | null = null;
  private endpoint: IrohEndpoint | null = null;
  private endpointBinding: Promise<IrohEndpoint> | null = null;
  private readonly connections = new Map<string, ActiveConnection>();
  private readonly statuses = new Map<string, DeviceConnectionStatus>();
  private readonly listeners = new Set<() => void>();
  private generation = 0;
  private disposed = false;

  constructor(deps: DeviceConnectionsDependencies) {
    this.deps = { profile: 'production', now: () => new Date().toISOString(), ...deps };
  }

  status(nodeId: string): DeviceConnectionStatus {
    return this.statuses.get(nodeId) ?? { state: 'disconnected' };
  }

  isConnected(nodeId: string): boolean {
    return this.status(nodeId).state === 'connected';
  }

  onDidChange(listener: () => void): Disposable {
    this.listeners.add(listener);
    return toDisposable(() => this.listeners.delete(listener));
  }

  /** Connects to a paired device by its `nodeId`. Idempotent while already
   * connected or connecting. */
  async connect(nodeId: string): Promise<void> {
    if (this.disposed) throw new Error('the connection manager is disposed');
    const device = this.deps.store.get(nodeId);
    if (!device) throw new Error(`unknown device ${nodeId}; pair it first`);

    const current = this.status(nodeId).state;
    if (current === 'connected' || current === 'connecting') return;

    this.setStatus(nodeId, { state: 'connecting' });
    try {
      const [module, endpoint] = await this.ensureEndpoint();
      const addr = module.EndpointTicket.fromString(device.ticket).endpointAddr();
      const connection = await endpoint.connect(
        addr,
        Array.from(new TextEncoder().encode(ALPN_TERMINAL)),
      );

      this.generation += 1;
      const generation = this.generation;
      this.connections.set(nodeId, { connection, generation });
      this.deps.store.markConnected(nodeId, this.deps.now());
      this.setStatus(nodeId, { state: 'connected' });
      this.watchClosed(nodeId, connection, generation);
    } catch (error) {
      this.deps.store.setOnline(nodeId, false);
      this.setStatus(nodeId, {
        state: 'error',
        code: 'CONNECT_FAILED',
        message: error instanceof Error ? error.message : String(error),
      });
      throw error;
    }
  }

  disconnect(nodeId: string): void {
    const active = this.connections.get(nodeId);
    if (!active) return;
    this.connections.delete(nodeId);
    this.deps.store.setOnline(nodeId, false);
    this.setStatus(nodeId, { state: 'disconnected' });
    try {
      active.connection.close(BigInt(0), Array.from(new TextEncoder().encode('user disconnect')));
    } catch {
      // Already gone - disconnecting was the goal.
    }
  }

  /**
   * A transport for one new terminal session on the device's connection
   * (doc 8.2: every session is its own bi-stream).
   */
  createTerminalTransport(nodeId: string): TerminalStreamTransport {
    const active = this.connections.get(nodeId);
    if (!active) throw new Error('connect to the device before opening a terminal');
    return new TerminalStreamTransport(terminalStreamFactory(active.connection));
  }

  async dispose(): Promise<void> {
    this.disposed = true;
    for (const nodeId of [...this.connections.keys()]) this.disconnect(nodeId);
    this.listeners.clear();
    const endpoint = this.endpoint;
    this.endpoint = null;
    this.endpointBinding = null;
    if (endpoint) await endpoint.close();
  }

  private async ensureEndpoint(): Promise<[IrohModule, IrohEndpoint]> {
    if (this.module && this.endpoint) return [this.module, this.endpoint];
    if (!this.endpointBinding) {
      this.endpointBinding = (async () => {
        const module = await this.deps.loadIroh();
        this.module = module;

        const builder = module.Endpoint.builder();
        if (this.deps.profile === 'loopback') {
          module.presetMinimal(builder);
          builder.relayMode(module.RelayMode.disabled());
          builder.bindAddr('127.0.0.1:0');
        } else {
          module.presetN0(builder);
        }

        const secret =
          this.deps.identitySeed !== null
            ? module.SecretKey.fromBytes(this.deps.identitySeed)
            : module.SecretKey.generate();
        if (this.deps.identitySeed === null) {
          this.deps.onIdentityCreated(secret.toBytes());
        }
        builder.secretKey(secret.toBytes());

        const endpoint = await builder.bind();
        this.endpoint = endpoint;
        return endpoint;
      })();
      // A failed bind must not poison every later attempt.
      this.endpointBinding.catch(() => {
        this.endpointBinding = null;
      });
    }
    const endpoint = await this.endpointBinding;
    if (!this.module) throw new Error('iroh module vanished during bind');
    return [this.module, endpoint];
  }

  /** Marks the device offline when its connection dies underneath us. The
   * agent closes refused controllers with CONTROLLER_ALREADY_CONNECTED
   * (doc 7.7) - that lands here too, surfaced as an error status. */
  private watchClosed(nodeId: string, connection: IrohConnection, generation: number): void {
    void connection
      .closed()
      .catch((error: unknown) => String(error))
      .then((reason) => {
        const active = this.connections.get(nodeId);
        if (!active || active.generation !== generation) return;
        this.connections.delete(nodeId);
        this.deps.store.setOnline(nodeId, false);
        if (String(reason).includes('CONTROLLER_ALREADY_CONNECTED')) {
          this.setStatus(nodeId, {
            state: 'error',
            code: 'CONTROLLER_ALREADY_CONNECTED',
            message: String(reason),
          });
        } else {
          this.setStatus(nodeId, { state: 'disconnected' });
        }
      });
  }

  private setStatus(nodeId: string, status: DeviceConnectionStatus): void {
    this.statuses.set(nodeId, status);
    for (const listener of this.listeners) listener();
  }
}
