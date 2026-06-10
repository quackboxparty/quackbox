import { Schema } from 'effect';

const SLUG_INNER = '[a-z0-9][a-z0-9_]*';
const TAG_CATEGORY_ALT = '(subject|difficulty|audience|region|format|warning)';

// BCP-47-ish: `en`, `de`, `en-US`. Strict so typos like `EN` or `de_DE` fail.
export const LocaleCode = Schema.String.check(Schema.isPattern(/^[a-z]{2}(-[A-Z]{2})?$/));
export type LocaleCode = typeof LocaleCode.Type;

// Canonical slug pattern — starts with letter or digit, then alphanum/underscore.
// Used as the basis for all ID schemas and tag refs.
export const Slug = Schema.String.check(Schema.isPattern(new RegExp(`^${SLUG_INNER}$`)));
export type Slug = typeof Slug.Type;

export const QuestionId = Schema.String.check(Schema.isPattern(new RegExp(`^q_${SLUG_INNER}$`)));
export type QuestionId = typeof QuestionId.Type;

export const PackId = Schema.String.check(Schema.isPattern(new RegExp(`^pack_${SLUG_INNER}$`)));
export type PackId = typeof PackId.Type;

export const BoardId = Schema.String.check(Schema.isPattern(new RegExp(`^board_${SLUG_INNER}$`)));
export type BoardId = typeof BoardId.Type;

export const GamemodeId = Slug;
export type GamemodeId = typeof GamemodeId.Type;

/** Local ID — bare slug inside a question (choice id, order-item, board slot key). */
export const LocalId = Slug;
export type LocalId = typeof LocalId.Type;

// Closed enum of tag categories. Adding one is a schema PR.
export const TagCategory = Schema.Literals([
	'subject',
	'difficulty',
	'audience',
	'region',
	'format',
	'warning'
]);
export type TagCategory = typeof TagCategory.Type;

export const TagRef = Schema.String.check(
	Schema.isPattern(new RegExp(`^${TAG_CATEGORY_ALT}:${SLUG_INNER}$`))
);
export type TagRef = typeof TagRef.Type;

export function tagRefFor(category: typeof TagCategory.Type) {
	return Schema.String.check(Schema.isPattern(new RegExp(`^${category}:${SLUG_INNER}$`)));
}

// SPDX allowlist; small on purpose, expand via schema PR.
export const License = Schema.Literals([
	'CC0-1.0',
	'CC-BY-4.0',
	'CC-BY-SA-4.0',
	'CC-BY-NC-4.0',
	'CC-BY-ND-4.0',
	'MIT'
]);
export type License = typeof License.Type;

export const Source = Schema.Struct({
	// ISO date `YYYY-MM-DD`; full datetime would be overkill for a citation.
	accessed: Schema.optionalKey(Schema.String.check(Schema.isPattern(/^\d{4}-\d{2}-\d{2}$/))),
	note: Schema.optionalKey(Schema.String),
	url: Schema.String.check(Schema.isPattern(/^https?:\/\//))
});
export type Source = typeof Source.Type;

export const Deprecation = Schema.Struct({
	reason: Schema.String,
	replaced_by: Schema.optionalKey(QuestionId)
});
export type Deprecation = typeof Deprecation.Type;
