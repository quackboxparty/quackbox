import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

/**
 * Repo root. Uses process.cwd() when running from scripts, falling back
 * to module path resolution for runtime/SvelteKit environments.
 */
export const REPO_ROOT = process.cwd().endsWith('quackbox')
	? process.cwd()
	: resolve(dirname(fileURLToPath(import.meta.url)), '..', '..', '..', '..');

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
