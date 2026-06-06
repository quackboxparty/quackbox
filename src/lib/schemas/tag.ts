import * as v from 'valibot';
import { LocaleCode, TagCategory, tagRefFor } from './common.ts';

/** Single entry in a tag registry file. `id` is constrained per-category. */
function tagRegistryEntry<C extends v.InferOutput<typeof TagCategory>>(category: C) {
	return v.strictObject({
		id: tagRefFor(category),
		default_lang: LocaleCode,
		label: v.string(),
		description: v.optional(v.string())
	});
}

/** Canonical registry file: one category per file, all entries share the prefix. */
export function tagRegistryFile<C extends v.InferOutput<typeof TagCategory>>(category: C) {
	return v.array(tagRegistryEntry(category));
}

export const TagRegistryFiles = {
	subject: tagRegistryFile('subject'),
	difficulty: tagRegistryFile('difficulty'),
	audience: tagRegistryFile('audience'),
	region: tagRegistryFile('region'),
	format: tagRegistryFile('format'),
	warning: tagRegistryFile('warning')
} as const;

/** Overlay entry: only translatable fields. `default_lang` is canonical-only. */
function tagOverlayEntry<C extends v.InferOutput<typeof TagCategory>>(category: C) {
	return v.strictObject({
		id: tagRefFor(category),
		label: v.optional(v.string()),
		description: v.optional(v.string())
	});
}

export function tagOverlayFile<C extends v.InferOutput<typeof TagCategory>>(category: C) {
	return v.array(tagOverlayEntry(category));
}

export const TagOverlayFiles = {
	subject: tagOverlayFile('subject'),
	difficulty: tagOverlayFile('difficulty'),
	audience: tagOverlayFile('audience'),
	region: tagOverlayFile('region'),
	format: tagOverlayFile('format'),
	warning: tagOverlayFile('warning')
} as const;
