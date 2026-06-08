import { GamemodeManifest } from '$lib/schemas/gamemode';
import * as v from 'valibot';
import { describe, expect, it } from 'vitest';

const BASE = {
	accepts: { kinds: ['text'] as const, variants: ['multiple_choice'] as const },
	id: 'classic',
	name: 'Classic Quiz',
	ui: { host_view: 'Host.svelte', player_view: 'Player.svelte' }
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
				requires: { min_players: 2, timer: true },
				ui: {
					host_view: 'Host.svelte',
					player_view: 'Player.svelte',
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
				max_choices: 4,
				min_choices: 6
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
