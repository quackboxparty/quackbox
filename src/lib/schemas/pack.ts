import { Schema } from 'effect';

import { GamemodeId, License, LocaleCode, PackId, QuestionId, TagRef } from './common.ts';
import { QuestionKind, VariantName } from './question.ts';

export const PackFilter = Schema.Struct({
	kinds: Schema.optionalKey(Schema.Array(QuestionKind)),
	limit: Schema.optionalKey(Schema.Number.check(Schema.isInt(), Schema.isGreaterThanOrEqualTo(1))),
	tags_all: Schema.optionalKey(Schema.Array(TagRef)),
	tags_any: Schema.optionalKey(Schema.Array(TagRef)),
	tags_none: Schema.optionalKey(Schema.Array(TagRef)),
	variants_any: Schema.optionalKey(Schema.Array(VariantName))
});
export type PackFilter = typeof PackFilter.Type;

export const Pack = Schema.Struct({
	author: Schema.optionalKey(Schema.String),
	default_lang: Schema.optionalKey(LocaleCode),
	description: Schema.optionalKey(Schema.String),
	filter: Schema.optionalKey(PackFilter),
	id: PackId,
	includes: Schema.optionalKey(Schema.Array(PackId)),
	license: Schema.optionalKey(License),
	questions: Schema.optionalKey(Schema.Array(QuestionId)),
	recommended_gamemodes: Schema.optionalKey(Schema.Array(GamemodeId)),
	title: Schema.String
}).check(
	Schema.makeFilter((p) =>
		(p.includes !== undefined && p.includes.length > 0) ||
		(p.questions !== undefined && p.questions.length > 0) ||
		p.filter !== undefined
			? undefined
			: 'pack must define at least one of: includes, questions, filter'
	)
);
export type Pack = typeof Pack.Type;

// Pack files are single-pack-per-file (unlike question files which are arrays).
export const PackFile = Pack;
export type PackFile = typeof PackFile.Type;

export const PackOverlay = Schema.Struct({
	description: Schema.optionalKey(Schema.String),
	id: PackId,
	title: Schema.optionalKey(Schema.String)
});
export type PackOverlay = typeof PackOverlay.Type;

export const PackOverlayFile = PackOverlay;
export type PackOverlayFile = typeof PackOverlayFile.Type;
