import * as v from 'valibot';

import { GamemodeId, License, LocaleCode, PackId, QuestionId, TagRef } from './common.ts';
import { QuestionKind, VariantName } from './question.ts';

export const PackFilter = v.strictObject({
	kinds: v.optional(v.array(QuestionKind)),
	limit: v.optional(v.pipe(v.number(), v.integer(), v.minValue(1))),
	tags_all: v.optional(v.array(TagRef)),
	tags_any: v.optional(v.array(TagRef)),
	tags_none: v.optional(v.array(TagRef)),
	variants_any: v.optional(v.array(VariantName))
});
export type PackFilter = v.InferOutput<typeof PackFilter>;

export const Pack = v.pipe(
	v.strictObject({
		author: v.optional(v.string()),
		default_lang: v.optional(LocaleCode),
		description: v.optional(v.string()),
		filter: v.optional(PackFilter),
		id: PackId,
		includes: v.optional(v.array(PackId)),
		license: v.optional(License),
		questions: v.optional(v.array(QuestionId)),
		recommended_gamemodes: v.optional(v.array(GamemodeId)),
		title: v.string()
	}),
	v.check(
		(p) =>
			(p.includes && p.includes.length > 0) ??
			(p.questions && p.questions.length > 0) ??
			p.filter !== undefined,
		'pack must define at least one of: includes, questions, filter'
	)
);
export type Pack = v.InferOutput<typeof Pack>;

// Pack files are single-pack-per-file (unlike question files which are arrays).
export const PackFile = Pack;
export type PackFile = v.InferOutput<typeof PackFile>;

export const PackOverlay = v.strictObject({
	description: v.optional(v.string()),
	id: PackId,
	title: v.optional(v.string())
});
export type PackOverlay = v.InferOutput<typeof PackOverlay>;

export const PackOverlayFile = PackOverlay;
export type PackOverlayFile = v.InferOutput<typeof PackOverlayFile>;
