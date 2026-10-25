import {
  checkFailed, checked, deferred, failed, installStarted, installed, progressed, started,
} from "./updateHandlers";
import type { UpdateEvent, UpdateState } from "./updateTypes";

export { IDLE, busy, versionOf } from "./updateTypes";
export type { Retry, UpdateEvent, UpdateState } from "./updateTypes";

type Handlers = {
  [T in UpdateEvent["type"]]: (state: UpdateState, event: Extract<UpdateEvent, { type: T }>) => UpdateState;
};

const HANDLERS: Handlers = {
  "check-started": started,
  checked: (state, event) => checked(state, event.result),
  "check-failed": checkFailed,
  "install-started": installStarted,
  progress: progressed,
  installed,
  failed,
  later: deferred,
};

/** Applies one event to the update state; returns the same object when nothing changes. */
export function reduce(state: UpdateState, event: UpdateEvent): UpdateState {
  const handler = HANDLERS[event.type] as (state: UpdateState, event: UpdateEvent) => UpdateState;
  return handler(state, event);
}
