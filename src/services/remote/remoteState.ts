/**
 * Remote-mode UI state machine (doc 5.3).
 *
 * Kept as a pure reducer so the table in the doc can be checked directly, rather
 * than being spread across view callbacks where an unreachable state is easy to
 * miss.
 */

export type RemoteState =
  | 'LocalMode'
  | 'RemoteIdle'
  | 'Connecting'
  | 'Connected'
  | 'Transferring'
  | 'Error';

export type RemoteEvent =
  | { type: 'switchToRemote' }
  | { type: 'switchToLocal' }
  | { type: 'connect' }
  | { type: 'opened' }
  | { type: 'openFailed' }
  | { type: 'dropNote' }
  | { type: 'transferFinished' }
  | { type: 'disconnect' }
  | { type: 'connectionLost' }
  | { type: 'chooseDevice' };

export interface Capabilities {
  /** Whether keystrokes reach a shell. */
  input: boolean;
  /** Whether dropping a note starts a transfer. */
  drop: boolean;
  /** Whether the device picker is usable. */
  deviceSelection: boolean;
}

/** Doc 5.3's capability table, verbatim. */
export function capabilities(state: RemoteState): Capabilities {
  switch (state) {
    case 'LocalMode':
      return { input: true, drop: true, deviceSelection: false };
    case 'RemoteIdle':
      return { input: false, drop: false, deviceSelection: true };
    case 'Connecting':
      return { input: false, drop: false, deviceSelection: false };
    case 'Connected':
      return { input: true, drop: true, deviceSelection: false };
    case 'Transferring':
      // Input stays live during a transfer: the credit window keeps file traffic
      // from starving the terminal, so there is no reason to freeze it.
      return { input: true, drop: false, deviceSelection: false };
    case 'Error':
      return { input: false, drop: false, deviceSelection: true };
  }
}

export function transition(state: RemoteState, event: RemoteEvent): RemoteState {
  switch (state) {
    case 'LocalMode':
      return event.type === 'switchToRemote' ? 'RemoteIdle' : state;

    case 'RemoteIdle':
      if (event.type === 'connect') return 'Connecting';
      if (event.type === 'switchToLocal') return 'LocalMode';
      return state;

    case 'Connecting':
      if (event.type === 'opened') return 'Connected';
      if (event.type === 'openFailed' || event.type === 'connectionLost') return 'Error';
      return state;

    case 'Connected':
      if (event.type === 'dropNote') return 'Transferring';
      if (event.type === 'disconnect') return 'RemoteIdle';
      if (event.type === 'connectionLost') return 'Error';
      if (event.type === 'switchToLocal') return 'LocalMode';
      return state;

    case 'Transferring':
      if (event.type === 'transferFinished') return 'Connected';
      if (event.type === 'connectionLost') return 'Error';
      if (event.type === 'disconnect') return 'RemoteIdle';
      if (event.type === 'switchToLocal') return 'LocalMode';
      return state;

    case 'Error':
      if (event.type === 'connect') return 'Connecting';
      if (event.type === 'chooseDevice') return 'RemoteIdle';
      if (event.type === 'switchToLocal') return 'LocalMode';
      return state;
  }
}

/** Every state the machine can reach from LocalMode. */
export function reachableStates(): Set<RemoteState> {
  const all: RemoteEvent[] = [
    { type: 'switchToRemote' },
    { type: 'switchToLocal' },
    { type: 'connect' },
    { type: 'opened' },
    { type: 'openFailed' },
    { type: 'dropNote' },
    { type: 'transferFinished' },
    { type: 'disconnect' },
    { type: 'connectionLost' },
    { type: 'chooseDevice' },
  ];

  const seen = new Set<RemoteState>(['LocalMode']);
  const queue: RemoteState[] = ['LocalMode'];

  while (queue.length > 0) {
    const current = queue.shift() as RemoteState;
    for (const event of all) {
      const next = transition(current, event);
      if (!seen.has(next)) {
        seen.add(next);
        queue.push(next);
      }
    }
  }

  return seen;
}
