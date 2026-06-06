import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

/**
 * Repo root, resolved relative to this file. Keeps the loader usable both
 * from `scripts/` and from runtime code without an env var.
 */
export const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..', '..', '..');

export const DATA_DIR = join(REPO_ROOT, 'data');

export const TAG_CATEGORIES = [
	'subject',
	'difficulty',
	'audience',
	'region',
	'format',
	'warning'
] as const;
export type TagCategoryName = (typeof TAG_CATEGORIES)[number];
