import { describe, expect, it } from 'vitest';
import * as v from 'valibot';
import { GamemodeManifest } from '$lib/schemas/gamemode';

const BASE = {
	id: 'classic',
	name: 'Classic Quiz',
	accepts: { kinds: ['text'] as const, variants: ['multiple_choice'] as const },
	ui: { player_view: 'Player.svelte', host_view: 'Host.svelte' }
};

describe('gamemode', () => {
	it('accepts valid manifest', () => {
		expect(v.safeParse(GamemodeManifest, BASE).success).toBe(true);
	});

	it('accepts manifest with optional fields', () => {
		expect(
			v.safeParse(GamemodeManifest, {
				...BASE,
				description: 'Traditional style.',
				requires: { timer: true, min_players: 2 },
				ui: {
					player_view: 'Player.svelte',
					host_view: 'Host.svelte',
					spectator_view: 'Spectator.svelte'
				}
			}).success
		).toBe(true);
	});

	it('rejects max_choices < min_choices', () => {
		const bad = {
			...BASE,
			accepts: {
				...BASE.accepts,
				min_choices: 6,
				max_choices: 4
			}
		};
		expect(v.safeParse(GamemodeManifest, bad).success).toBe(false);
	});

	it('rejects empty kinds or variants', () => {
		expect(
			v.safeParse(GamemodeManifest, {
				...BASE,
				accepts: { kinds: [] as const, variants: ['multiple_choice'] as const }
			}).success
		).toBe(false);
	});

	it('rejects non-svelte ui file', () => {
		expect(
			v.safeParse(GamemodeManifest, {
				...BASE,
				ui: { ...BASE.ui, player_view: 'Player.vue' }
			}).success
		).toBe(false);
	});
});
