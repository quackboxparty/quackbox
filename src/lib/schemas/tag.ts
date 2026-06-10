import * as v from 'valibot';

import { LocaleCode, type TagCategory, tagRefFor } from './common.ts';

type TagCategoryValue = v.InferOutput<typeof TagCategory>;

/** Canonical registry file: one category per file, all entries share the prefix. */
export function tagRegistryFile(category: TagCategoryValue) {
	return v.array(tagRegistryEntry(category));
}

/** Single entry in a tag registry file. `id` is constrained per-category. */
export function tagRegistryEntry(category: TagCategoryValue) {
	return v.strictObject({
		default_lang: LocaleCode,
		description: v.optional(v.string()),
		id: tagRefFor(category),
		label: v.string()
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
	return v.array(tagOverlayEntry(category));
}

/** Overlay entry: only translatable fields. `default_lang` is canonical-only. */
export function tagOverlayEntry(category: TagCategoryValue) {
	return v.strictObject({
		description: v.optional(v.string()),
		id: tagRefFor(category),
		label: v.optional(v.string())
	});
}

export type Tag = v.InferOutput<ReturnType<typeof tagRegistryEntry>>;
export type TagOverlay = v.InferOutput<ReturnType<typeof tagOverlayEntry>>;

export const TagOverlayFiles = {
	audience: tagOverlayFile('audience'),
	difficulty: tagOverlayFile('difficulty'),
	format: tagOverlayFile('format'),
	region: tagOverlayFile('region'),
	subject: tagOverlayFile('subject'),
	warning: tagOverlayFile('warning')
} as const;
