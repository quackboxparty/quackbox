import * as v from 'valibot';

// BCP-47-ish: `en`, `de`, `en-US`. Strict so typos like `EN` or `de_DE` fail.
export const LocaleCode = v.pipe(v.string(), v.regex(/^[a-z]{2}(-[A-Z]{2})?$/));
export type LocaleCode = v.InferOutput<typeof LocaleCode>;

// Canonical slug pattern — starts with letter or digit, then alphanum/underscore.
// Used as the basis for all ID schemas and tag refs.
const slugInner = '[a-z0-9][a-z0-9_]*';

// Bare slug — used for gamemode ids.
export const Slug = v.pipe(v.string(), v.regex(new RegExp(`^${slugInner}$`)));
export type Slug = v.InferOutput<typeof Slug>;

export const QuestionId = v.pipe(v.string(), v.regex(new RegExp(`^q_${slugInner}$`)));
export type QuestionId = v.InferOutput<typeof QuestionId>;

export const PackId = v.pipe(v.string(), v.regex(new RegExp(`^pack_${slugInner}$`)));
export type PackId = v.InferOutput<typeof PackId>;

export const BoardId = v.pipe(v.string(), v.regex(new RegExp(`^board_${slugInner}$`)));
export type BoardId = v.InferOutput<typeof BoardId>;

export const GamemodeId = Slug;
export type GamemodeId = v.InferOutput<typeof GamemodeId>;

/** Local ID — bare slug inside a question (choice id, order-item, board slot key). */
export const LocalId = Slug;
export type LocalId = v.InferOutput<typeof LocalId>;

// Closed enum of tag categories. Adding one is a schema PR.
export const TagCategory = v.picklist([
	'subject',
	'difficulty',
	'audience',
	'region',
	'format',
	'warning'
]);
export type TagCategory = v.InferOutput<typeof TagCategory>;

const tagCategoryAlt = '(subject|difficulty|audience|region|format|warning)';
export const TagRef = v.pipe(v.string(), v.regex(new RegExp(`^${tagCategoryAlt}:${slugInner}$`)));
export type TagRef = v.InferOutput<typeof TagRef>;

/** Build a TagRef restricted to a single category — used by per-category registries. */
export function tagRefFor(category: TagCategory) {
	return v.pipe(v.string(), v.regex(new RegExp(`^${category}:${slugInner}$`)));
}

// SPDX allowlist; small on purpose, expand via schema PR.
export const License = v.picklist([
	'CC0-1.0',
	'CC-BY-4.0',
	'CC-BY-SA-4.0',
	'CC-BY-NC-4.0',
	'CC-BY-ND-4.0',
	'MIT'
]);
export type License = v.InferOutput<typeof License>;

export const Source = v.strictObject({
	// ISO date `YYYY-MM-DD`; full datetime would be overkill for a citation.
	accessed: v.optional(v.pipe(v.string(), v.regex(/^\d{4}-\d{2}-\d{2}$/))),
	note: v.optional(v.string()),
	url: v.pipe(v.string(), v.url())
});
export type Source = v.InferOutput<typeof Source>;

export const Deprecation = v.strictObject({
	reason: v.string(),
	replaced_by: v.optional(QuestionId)
});
export type Deprecation = v.InferOutput<typeof Deprecation>;
