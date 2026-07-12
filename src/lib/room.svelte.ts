import type { ClientView, Command } from './bindings/Protocol';
import type { Grant } from './bindings/Grants';

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

// Own slot (grants, score, connected), or null before Join resolves.
export function me() {
	return room.gamestate?.players[room.player ?? ''] ?? null;
}

/** Grant check for templates: `{#if has('Moderate')}…`. Reads $state, reactive. */
export function has(g: Grant): boolean {
	return (room.gamestate?.players[room.player ?? '']?.grants ?? []).includes(g);
}
