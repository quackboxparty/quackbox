import * as v from 'valibot';
import { GamemodeId, License, LocaleCode, PackId, QuestionId, TagRef } from './common.ts';
import { QuestionKind, VariantName } from './question.ts';

const PackFilter = v.strictObject({
	tags_all: v.optional(v.array(TagRef)),
	tags_any: v.optional(v.array(TagRef)),
	tags_none: v.optional(v.array(TagRef)),
	kinds: v.optional(v.array(QuestionKind)),
	variants_any: v.optional(v.array(VariantName)),
	limit: v.optional(v.pipe(v.number(), v.integer(), v.minValue(1)))
});

export const Pack = v.pipe(
	v.strictObject({
		id: PackId,
		title: v.string(),
		description: v.optional(v.string()),
		author: v.optional(v.string()),
		license: v.optional(License),
		default_lang: v.optional(LocaleCode),
		recommended_gamemodes: v.optional(v.array(GamemodeId)),
		includes: v.optional(v.array(PackId)),
		questions: v.optional(v.array(QuestionId)),
		filter: v.optional(PackFilter)
	}),
	v.check(
		(p) =>
			(p.includes && p.includes.length > 0) ||
			(p.questions && p.questions.length > 0) ||
			p.filter !== undefined,
		'pack must define at least one of: includes, questions, filter'
	)
);
export type Pack = v.InferOutput<typeof Pack>;

// Pack files are single-pack-per-file (unlike question files which are arrays).
export const PackFile = Pack;
export type PackFile = v.InferOutput<typeof PackFile>;

export const PackOverlay = v.strictObject({
	id: PackId,
	title: v.optional(v.string()),
	description: v.optional(v.string())
});
export type PackOverlay = v.InferOutput<typeof PackOverlay>;

export const PackOverlayFile = PackOverlay;
export type PackOverlayFile = v.InferOutput<typeof PackOverlayFile>;
