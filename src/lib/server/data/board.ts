import type * as Effect from 'effect/Effect';
import type * as FileSystem from 'effect/FileSystem';
import { BoardFile } from '../../schemas/index.ts';
import type { BoardCategory } from '../../schemas/index.ts';

import type { LoadedDataset, LoadIssue } from './shared.ts';
import { parse } from './util.ts';

/** A resolved 2D grid — `grid[categoryIndex][pointIndex] = questionId | null`. */
export type BoardGrid = (null | string)[][];

export interface BuildBoardOptions {
	locale: string;
	seed?: number;
}

/**
 * Load and validate a board YAML file. The caller passes the full path —
 * keeps `loadBoard` free of the `Path` service and matches the loader
 * convention of taking absolute paths.
 */
export function loadBoard(
	file: string
): Effect.Effect<BoardFile, LoadIssue[], FileSystem.FileSystem> {
	return parse(file, BoardFile);
}

/**
 * Build a resolved NxM board. Returns `grid[ci][pi] = questionId | null`.
 * Unresolvable slots are `null` and reported as issues.
 */
export function buildBoard(
	ds: LoadedDataset,
	board: BoardFile,
	opts: BuildBoardOptions
): { grid: BoardGrid; issues: LoadIssue[] } {
	const issues: LoadIssue[] = [];
	const grid: BoardGrid = [];
	const rng = mulberry32(opts.seed ?? 42);
	const diffMap = board.difficulty_map
		? Object.fromEntries(
				Object.entries(board.difficulty_map).map(([k, v]) => [
					k,
					Array.isArray(v) ? v : Array.from(v)
				])
			)
		: {};

	const packCache = new Map<string, string[]>();
	const used = new Set<string>();

	for (const cat of board.categories) {
		const row: (null | string)[] = [];
		for (const point of board.points.entries()) {
			const pointKey = String(point);

			const explicit = cat.question_ids?.[pointKey];
			if (explicit) {
				row.push(explicit);
				used.add(explicit);
				continue;
			}

			const candidates = Array.isArray(cat.question_ids?.[pointKey])
				? []
				: buildCandidates(cat, pointKey, ds, packCache, diffMap);
			const unused = candidates.filter((id) => !used.has(id));
			const pool = unused.length > 0 ? unused : candidates;

			const [picked = null] = pool.length > 0 ? shuffle(pool, rng) : [];
			row.push(picked);
			if (picked) used.add(picked);
		}
		grid.push(row);
	}

	return { grid, issues };
}

function shuffle<T>(arr: T[], rng: () => number): T[] {
	const out = [...arr];
	for (let i = out.length - 1; i > 0; i--) {
		const j = Math.floor(rng() * (i + 1));
		const temp = out[i];
		if (out[j] != null) out[i] = out[j];
		if (temp != null) out[j] = temp;
	}
	return out;
}

/** Deterministic seeded PRNG (mulberry32). */
function mulberry32(a: number) {
	return () => {
		a |= 0;
		a = (a + 0x6d2b79f5) | 0;
		let t = Math.imul(a ^ (a >>> 15), 1 | a);
		t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
		return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
	};
}

function buildCandidates(
	cat: BoardCategory,
	pointKey: string,
	ds: LoadedDataset,
	packCache: Map<string, string[]>,
	diffMap: Record<string, string[] | undefined>
): string[] {
	let candidates: string[] = [];

	if (cat.pack_ref) {
		candidates = resolvePack(ds, packCache, cat.pack_ref);
	}

	if (cat.filter) {
		const filteredSet = new Set(queryPool(ds, cat.filter));
		candidates =
			candidates.length > 0 ? candidates.filter((qid) => filteredSet.has(qid)) : [...filteredSet];
	}

	const diffTags = diffMap[pointKey];
	if (diffTags && diffTags.length > 0) {
		candidates = candidates.filter((qid) => {
			const tags = ds.questions.get(qid)?.item.tags;
			return tags !== undefined && diffTags.some((tag) => tags.includes(tag));
		});
	}

	return candidates;
}

function resolvePack(
	ds: LoadedDataset,
	packCache: Map<string, string[]>,
	packId: string
): string[] {
	const cached = packCache.get(packId);
	if (cached !== undefined) return cached;

	const ids: string[] = [];
	const pack = ds.packs.get(packId);
	if (!pack) return ids;
	for (const incl of pack.item.includes ?? []) {
		ids.push(...resolvePack(ds, packCache, incl));
	}
	ids.push(...(pack.item.questions ?? []));
	if (pack.item.filter) {
		ids.push(...queryPool(ds, pack.item.filter));
	}
	const unique = [...new Set(ids)];
	packCache.set(packId, unique);
	return unique;
}

// Imported late to keep this file's bottom (effects, types) free of the
// query engine's pure function imports.
import { queryPool } from './query.ts';
