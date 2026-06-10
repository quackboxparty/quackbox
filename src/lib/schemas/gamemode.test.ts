import { decodeStrict } from '$lib/schemas/decode';
import { GamemodeManifest } from '$lib/schemas/gamemode';
import { describe, expect, it } from 'vitest';

const decode = decodeStrict;

const BASE = {
	accepts: { kinds: ['text'] as const, variants: ['multiple_choice'] as const },
	id: 'classic',
	name: 'Classic Quiz',
	ui: { host_view: 'Host.svelte', player_view: 'Player.svelte' }
};

describe('gamemode', () => {
	it('accepts valid manifest', () => {
		expect(decode(GamemodeManifest)(BASE)).toEqual(BASE);
	});

	it('accepts manifest with optional fields', () => {
		const input = {
			...BASE,
			description: 'Traditional style.',
			requires: { min_players: 2, timer: true },
			ui: {
				host_view: 'Host.svelte',
				player_view: 'Player.svelte',
				spectator_view: 'Spectator.svelte'
			}
		};
		expect(decode(GamemodeManifest)(input)).toEqual(input);
	});

	it('rejects max_choices < min_choices', () => {
		expect(() =>
			decode(GamemodeManifest)({
				...BASE,
				accepts: { ...BASE.accepts, max_choices: 4, min_choices: 6 }
			})
		).toThrow();
	});

	it('rejects empty kinds or variants', () => {
		expect(() =>
			decode(GamemodeManifest)({
				...BASE,
				accepts: { kinds: [] as const, variants: ['multiple_choice'] as const }
			})
		).toThrow();
	});

	it('rejects non-svelte ui file', () => {
		expect(() =>
			decode(GamemodeManifest)({
				...BASE,
				ui: { ...BASE.ui, player_view: 'Player.vue' }
			})
		).toThrow();
	});
});
