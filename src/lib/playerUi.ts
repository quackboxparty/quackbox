import type { PlayerView } from '$lib/bindings/Protocol';

// Avatar color palette, keyed off the themed tokens so themes recolor avatars
// for free. Hash -> one of these, stable per name.
const AVATAR_COLORS = ['primary', 'secondary', 'accent', 'success', 'warning', 'danger'] as const;

/** Deterministic avatar color for a player name — same name always same color. */
export function playerColor(name: string): string {
	let h = 0;
	for (let i = 0; i < name.length; i++) h = (h * 31 + name.charCodeAt(i)) >>> 0;
	return `var(--color-${AVATAR_COLORS[h % AVATAR_COLORS.length]})`;
}

/** First character of a name for avatar fallback, uppercased. */
export function playerInitial(name: string): string {
	return name.trim().charAt(0).toUpperCase() || '?';
}

/** Player entries sorted by score, highest first — for standings/scoreboard. */
export function sortedByScore(players: Record<string, PlayerView>): [string, PlayerView][] {
	return Object.entries(players).sort(([, a], [, b]) => b.score - a.score);
}
