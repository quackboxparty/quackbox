import type { ClientView, Command } from "./bindings/Protocol";

/**
 * Shared room state. The room page owns the WebSocket and writes snapshots
 * here; the AppShell reads it to render the players drawer. Reactive across
 * modules because $state signals are shared by reference.
 */
export const room = $state<{
  code: string | null;
  player: string | null;
  gamestate: ClientView | null;
  send: ((cmd: Command) => void) | null;
}>({
  code: null,
  player: null,
  gamestate: null,
  send: null
});

export function clearRoom(): void {
  room.code = null;
  room.player = null;
  room.gamestate = null;
  room.send = null;
}
