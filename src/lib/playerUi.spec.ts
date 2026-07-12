import { describe, expect, it } from 'vitest';

import { playerColor, playerInitial } from './playerUi';

describe('playerUi', () => {
	it('assigns a stable color per name', () => {
		expect(playerColor('Alice')).toBe(playerColor('Alice'));
	});

	it('emits a themed CSS var string', () => {
		expect(playerColor('Bob')).toMatch(
			/^var\(--color-(primary|secondary|accent|success|warning|danger)\)$/
		);
	});

	it('takes the trimmed, uppercased initial', () => {
		expect(playerInitial('  carol')).toBe('C');
		expect(playerInitial('')).toBe('?');
	});
});
