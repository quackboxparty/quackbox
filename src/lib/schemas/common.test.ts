import { describe, expect, it } from 'vitest';
import * as v from 'valibot';
import {
	GamemodeId,
	License,
	LocaleCode,
	PackId,
	QuestionId,
	TagCategory,
	TagRef,
	tagRefFor
} from '$lib/schemas/common';

describe('common', () => {
	it('accepts valid locale codes', () => {
		expect(v.safeParse(LocaleCode, 'en').success).toBe(true);
		expect(v.safeParse(LocaleCode, 'de').success).toBe(true);
		expect(v.safeParse(LocaleCode, 'en-US').success).toBe(true);
	});

	it('rejects invalid locale codes', () => {
		expect(v.safeParse(LocaleCode, 'EN').success).toBe(false);
		expect(v.safeParse(LocaleCode, 'de_DE').success).toBe(false);
		expect(v.safeParse(LocaleCode, 'en-us').success).toBe(false);
		expect(v.safeParse(LocaleCode, 'english').success).toBe(false);
	});

	it('accepts valid question ids', () => {
		expect(v.safeParse(QuestionId, 'q_france_capital').success).toBe(true);
		expect(v.safeParse(QuestionId, 'q_one2').success).toBe(true);
	});

	it('rejects invalid question ids', () => {
		expect(v.safeParse(QuestionId, 'france_capital').success).toBe(false);
		expect(v.safeParse(QuestionId, 'Q_upper').success).toBe(false);
		expect(v.safeParse(QuestionId, 'q_').success).toBe(false);
	});

	it('accepts valid pack ids', () => {
		expect(v.safeParse(PackId, 'pack_britpop').success).toBe(true);
	});

	it('rejects invalid pack ids', () => {
		expect(v.safeParse(PackId, 'britpop').success).toBe(false);
	});

	it('accepts valid tag refs', () => {
		expect(v.safeParse(TagRef, 'subject:chemistry').success).toBe(true);
		expect(v.safeParse(TagRef, 'warning:nsfw').success).toBe(true);
	});

	it('rejects invalid tag refs', () => {
		expect(v.safeParse(TagRef, 'custom:tag').success).toBe(false); // unknown category
		expect(v.safeParse(TagRef, 'SUBJECT:x').success).toBe(false);
		expect(v.safeParse(TagRef, 'tag').success).toBe(false);
	});

	it('enforces category-specific tag refs', () => {
		const subjectTag = tagRefFor('subject');
		expect(v.safeParse(subjectTag, 'subject:geo').success).toBe(true);
		expect(v.safeParse(subjectTag, 'difficulty:easy').success).toBe(false);
	});

	it('accepts valid license identifiers', () => {
		expect(v.safeParse(License, 'CC0-1.0').success).toBe(true);
		expect(v.safeParse(License, 'MIT').success).toBe(true);
	});

	it('rejects unknown license identifiers', () => {
		expect(v.safeParse(License, 'GPL-3.0').success).toBe(false);
	});

	it('accepts valid gamemode ids', () => {
		expect(v.safeParse(GamemodeId, 'classic').success).toBe(true);
		expect(v.safeParse(GamemodeId, 'battle_royale').success).toBe(true);
	});
});
