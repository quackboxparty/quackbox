import { decodeStrict } from '$lib/schemas/decode';
import { tagOverlayFile, tagRegistryFile } from '$lib/schemas/tag';
import { describe, expect, it } from 'vitest';

const decode = decodeStrict;

describe('tag registry', () => {
	it('accepts valid registry entry', () => {
		const input = [{ default_lang: 'en', id: 'subject:geo', label: 'Geography' }];
		expect(decode(tagRegistryFile('subject'))(input)).toEqual(input);
	});

	it('accepts registry entry with description', () => {
		const input = [
			{
				default_lang: 'en',
				description: 'Most people know this',
				id: 'difficulty:general',
				label: 'General'
			}
		];
		expect(decode(tagRegistryFile('difficulty'))(input)).toEqual(input);
	});

	it('rejects id with wrong prefix for category', () => {
		expect(() =>
			decode(tagRegistryFile('subject'))([
				{ default_lang: 'en', id: 'difficulty:easy', label: 'Easy' }
			])
		).toThrow();
	});

	it('rejects unknown category in id', () => {
		expect(() =>
			decode(tagRegistryFile('subject'))([
				{ default_lang: 'en', id: 'custom:thing', label: 'Thing' }
			])
		).toThrow();
	});
});

describe('tag overlay', () => {
	it('accepts valid overlay entry', () => {
		const input = [{ id: 'subject:geo', label: 'Geografie' }];
		expect(decode(tagOverlayFile('subject'))(input)).toEqual(input);
	});

	it('accepts partial overlay entry (label only)', () => {
		const input = [{ id: 'difficulty:general', label: 'Allgemeinwissen' }];
		expect(decode(tagOverlayFile('difficulty'))(input)).toEqual(input);
	});

	it('rejects default_lang on overlay (canonical only)', () => {
		expect(() =>
			decode(tagOverlayFile('subject'))([{ default_lang: 'de', id: 'subject:geo', label: 'Geo' }])
		).toThrow();
	});
});
