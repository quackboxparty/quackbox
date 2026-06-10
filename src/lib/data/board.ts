import { join } from 'node:path';

import type { LoadedDataset, LoadIssue } from './load.ts';
import type { ResultAsync } from 'neverthrow';

import { type BoardCategory, BoardFile } from '../schemas/board.ts';
import { parse } from './util.ts';
import { queryPool } from './query.ts';

/** A resolved 2D grid — `grid[categoryIndex][pointIndex] = questionId | null`. */
export type BoardGrid = (string | null)[][];

export interface BuildBoardOptions {
	locale: string;
	seed?: number;
}

/** Load and validate a board YAML file. Returns err with issues on failure. */
export function loadBoard(dir: string, filename: string): ResultAsync<BoardFile, LoadIssue[]> {
	return parse(join(dir, filename), BoardFile);
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
	const diffMap = board.difficulty_map ?? {};

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

			const candidates = buildCandidates(cat, pointKey, ds, packCache, diffMap);
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
	// includes
	for (const incl of pack.item.includes ?? []) {
		ids.push(...resolvePack(ds, packCache, incl));
	}
	// explicit questions
	ids.push(...(pack.item.questions ?? []));
	// pack filter query
	if (pack.item.filter) {
		ids.push(...queryPool(ds, pack.item.filter));
	}
	const unique = [...new Set(ids)];
	packCache.set(packId, unique);
	return unique;
}
