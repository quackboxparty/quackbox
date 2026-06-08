import { tagOverlayFile, tagRegistryFile } from '$lib/schemas/tag';
import * as v from 'valibot';
import { describe, expect, it } from 'vitest';

describe('tag registry', () => {
	it('accepts valid registry entry', () => {
		expect(
			v.safeParse(tagRegistryFile('subject'), [
				{ default_lang: 'en', id: 'subject:geo', label: 'Geography' }
			]).success
		).toBe(true);
	});

	it('accepts registry entry with description', () => {
		expect(
			v.safeParse(tagRegistryFile('difficulty'), [
				{
					default_lang: 'en',
					description: 'Most people know this',
					id: 'difficulty:general',
					label: 'General'
				}
			]).success
		).toBe(true);
	});

	it('rejects id with wrong prefix for category', () => {
		expect(
			v.safeParse(tagRegistryFile('subject'), [
				{ default_lang: 'en', id: 'difficulty:easy', label: 'Easy' }
			]).success
		).toBe(false);
	});

	it('rejects unknown category in id', () => {
		expect(
			v.safeParse(tagRegistryFile('subject'), [
				{ default_lang: 'en', id: 'custom:thing', label: 'Thing' }
			]).success
		).toBe(false);
	});
});

describe('tag overlay', () => {
	it('accepts valid overlay entry', () => {
		expect(
			v.safeParse(tagOverlayFile('subject'), [{ id: 'subject:geo', label: 'Geografie' }]).success
		).toBe(true);
	});

	it('accepts partial overlay entry (label only)', () => {
		expect(
			v.safeParse(tagOverlayFile('difficulty'), [
				{ id: 'difficulty:general', label: 'Allgemeinwissen' }
			]).success
		).toBe(true);
	});

	it('rejects default_lang on overlay (canonical only)', () => {
		expect(
			v.safeParse(tagOverlayFile('subject'), [
				{ default_lang: 'de', id: 'subject:geo', label: 'Geo' }
			]).success
		).toBe(false);
	});
});
