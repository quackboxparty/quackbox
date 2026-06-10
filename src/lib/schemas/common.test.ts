import {
	GamemodeId,
	License,
	LocaleCode,
	PackId,
	QuestionId,
	TagRef,
	tagRefFor
} from '$lib/schemas/common';
import { decodeStrict } from '$lib/schemas/decode';
import { describe, expect, it } from 'vitest';

const decode = decodeStrict;

describe('common', () => {
	it('accepts valid locale codes', () => {
		expect(decode(LocaleCode)('en')).toBe('en');
		expect(decode(LocaleCode)('de')).toBe('de');
		expect(decode(LocaleCode)('en-US')).toBe('en-US');
	});

	it('rejects invalid locale codes', () => {
		expect(() => decode(LocaleCode)('EN')).toThrow();
		expect(() => decode(LocaleCode)('de_DE')).toThrow();
		expect(() => decode(LocaleCode)('en-us')).toThrow();
		expect(() => decode(LocaleCode)('english')).toThrow();
	});

	it('accepts valid question ids', () => {
		expect(decode(QuestionId)('q_france_capital')).toBe('q_france_capital');
		expect(decode(QuestionId)('q_one2')).toBe('q_one2');
	});

	it('rejects invalid question ids', () => {
		expect(() => decode(QuestionId)('france_capital')).toThrow();
		expect(() => decode(QuestionId)('Q_upper')).toThrow();
		expect(() => decode(QuestionId)('q_')).toThrow();
	});

	it('accepts valid pack ids', () => {
		expect(decode(PackId)('pack_britpop')).toBe('pack_britpop');
	});

	it('rejects invalid pack ids', () => {
		expect(() => decode(PackId)('britpop')).toThrow();
	});

	it('accepts valid tag refs', () => {
		expect(decode(TagRef)('subject:chemistry')).toBe('subject:chemistry');
		expect(decode(TagRef)('warning:nsfw')).toBe('warning:nsfw');
	});

	it('rejects invalid tag refs', () => {
		expect(() => decode(TagRef)('custom:tag')).toThrow(); // unknown category
		expect(() => decode(TagRef)('SUBJECT:x')).toThrow();
		expect(() => decode(TagRef)('tag')).toThrow();
	});

	it('enforces category-specific tag refs', () => {
		const subjectTag = tagRefFor('subject');
		expect(decode(subjectTag)('subject:geo')).toBe('subject:geo');
		expect(() => decode(subjectTag)('difficulty:easy')).toThrow();
	});

	it('accepts valid license identifiers', () => {
		expect(decode(License)('CC0-1.0')).toBe('CC0-1.0');
		expect(decode(License)('MIT')).toBe('MIT');
	});

	it('rejects unknown license identifiers', () => {
		expect(() => decode(License)('GPL-3.0')).toThrow();
	});

	it('accepts valid gamemode ids', () => {
		expect(decode(GamemodeId)('classic')).toBe('classic');
		expect(decode(GamemodeId)('battle_royale')).toBe('battle_royale');
	});
});
