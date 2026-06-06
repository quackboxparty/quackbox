import { describe, expect, it } from 'vitest';
import * as v from 'valibot';
import {
	tagRegistryFile,
	tagOverlayFile
} from '$lib/schemas/tag';

describe('tag registry', () => {
	it('accepts valid registry entry', () => {
		expect(
			v.safeParse(tagRegistryFile('subject'), [
				{ id: 'subject:geo', default_lang: 'en', label: 'Geography' }
			]).success
		).toBe(true);
	});

	it('accepts registry entry with description', () => {
		expect(
			v.safeParse(tagRegistryFile('difficulty'), [
				{
					id: 'difficulty:general',
					default_lang: 'en',
					label: 'General',
					description: 'Most people know this'
				}
			]).success
		).toBe(true);
	});

	it('rejects id with wrong prefix for category', () => {
		expect(
			v.safeParse(tagRegistryFile('subject'), [
				{ id: 'difficulty:easy', default_lang: 'en', label: 'Easy' }
			]).success
		).toBe(false);
	});

	it('rejects unknown category in id', () => {
		expect(
			v.safeParse(tagRegistryFile('subject'), [
				{ id: 'custom:thing', default_lang: 'en', label: 'Thing' }
			]).success
		).toBe(false);
	});
});

describe('tag overlay', () => {
	it('accepts valid overlay entry', () => {
		expect(
			v.safeParse(tagOverlayFile('subject'), [
				{ id: 'subject:geo', label: 'Geografie' }
			]).success
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
				{ id: 'subject:geo', default_lang: 'de', label: 'Geo' }
			]).success
		).toBe(false);
	});
});
