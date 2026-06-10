import { Schema } from 'effect';

import { LocaleCode, type TagCategory, tagRefFor } from './common.ts';

type TagCategoryValue = typeof TagCategory.Type;

/** Canonical registry file: one category per file, all entries share the prefix. */
export function tagRegistryFile(category: TagCategoryValue) {
	return Schema.Array(tagRegistryEntry(category));
}

/** Single entry in a tag registry file. `id` is constrained per-category. */
export function tagRegistryEntry(category: TagCategoryValue) {
	return Schema.Struct({
		default_lang: LocaleCode,
		description: Schema.optionalKey(Schema.String),
		id: tagRefFor(category),
		label: Schema.String
	});
}

export const TagRegistryFiles = {
	audience: tagRegistryFile('audience'),
	difficulty: tagRegistryFile('difficulty'),
	format: tagRegistryFile('format'),
	region: tagRegistryFile('region'),
	subject: tagRegistryFile('subject'),
	warning: tagRegistryFile('warning')
} as const;

export function tagOverlayFile(category: TagCategoryValue) {
	return Schema.Array(tagOverlayEntry(category));
}

/** Overlay entry: only translatable fields. `default_lang` is canonical-only. */
export function tagOverlayEntry(category: TagCategoryValue) {
	return Schema.Struct({
		description: Schema.optionalKey(Schema.String),
		id: tagRefFor(category),
		label: Schema.optionalKey(Schema.String)
	});
}

type TagEntrySchema = ReturnType<typeof tagRegistryEntry>;
export type Tag = TagEntrySchema extends Schema.Top ? TagEntrySchema['Type'] : never;

type TagOverlaySchema = ReturnType<typeof tagOverlayEntry>;
export type TagOverlay = TagOverlaySchema extends Schema.Top ? TagOverlaySchema['Type'] : never;

export const TagOverlayFiles = {
	audience: tagOverlayFile('audience'),
	difficulty: tagOverlayFile('difficulty'),
	format: tagOverlayFile('format'),
	region: tagOverlayFile('region'),
	subject: tagOverlayFile('subject'),
	warning: tagOverlayFile('warning')
} as const;
