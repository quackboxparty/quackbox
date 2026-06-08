import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
import * as v from 'valibot';
import { parse as parseYaml } from 'yaml';

import type { LoadedDataset, LoadIssue } from './load.ts';

import { type BoardCategory, BoardFile } from '../schemas/board.ts';
import { buildIndex, queryPool, type QuestionIndex } from './query.ts';

/** A resolved 2D grid — `grid[categoryIndex][pointIndex] = questionId | null`. */
export type BoardGrid = (null | string)[][];

export interface BuildBoardOptions {
  locale: string;
  seed?: number;
}

export async function loadBoard(
  dir: string,
  filename: string,
  issues: LoadIssue[]
): Promise<BoardFile | null> {
  try {
    const text = await readFile(join(dir, filename), 'utf8');
    const raw: unknown = parseYaml(text);
    const result = v.safeParse(BoardFile, raw);
    if (!result.success) {
      for (const issue of result.issues) {
        issues.push({
          file: filename,
          message: issue.message,
          path: formatPath(issue)
        });
      }
      return null;
    }
    return result.output;
  } catch (err) {
    issues.push({ file: filename, message: `failed to parse board: ${(err as Error).message}` });
    return null;
  }
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
  const idx = buildIndex(ds);

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

      const candidates = buildCandidates(cat, pointKey, ds, packCache, idx, diffMap);
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

function formatPath(issue: v.BaseIssue<unknown>): string {
  if (!issue.path) return '';
  return issue.path
    .map((p) => {
      const key = (p as { key?: number | string }).key;
      if (typeof key === 'number') return `[${key}]`;
      if (typeof key === 'string') return `.${key}`;
      return '';
    })
    .join('');
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
  idx: QuestionIndex,
  diffMap: Record<string, string[] | undefined>
): string[] {
  let candidates: string[] = [];

  if (cat.pack_ref) {
    candidates = resolvePack(ds, packCache, idx, cat.pack_ref);
  }

  if (cat.filter) {
    const filteredSet = new Set(queryPool(ds, cat.filter, idx));
    candidates =
      candidates.length > 0 ? candidates.filter((qid) => filteredSet.has(qid)) : [...filteredSet];
  }

  const diffTags = diffMap[pointKey];
  if (diffTags && diffTags.length > 0) {
    candidates = candidates.filter((qid) => {
      const tags = idx.tags.get(qid);
      return tags !== undefined && diffTags.some((tag) => tags.has(tag));
    });
  }

  return candidates;
}

function resolvePack(
  ds: LoadedDataset,
  packCache: Map<string, string[]>,
  idx: QuestionIndex,
  packId: string
): string[] {
  const cached = packCache.get(packId);
  if (cached !== undefined) return cached;

  const ids: string[] = [];
  const pack = ds.packs.find((p) => p.item.id === packId);
  if (!pack) return ids;
  // includes
  for (const incl of pack.item.includes ?? []) {
    ids.push(...resolvePack(ds, packCache, idx, incl));
  }
  // explicit questions
  ids.push(...(pack.item.questions ?? []));
  // pack filter query
  if (pack.item.filter) {
    ids.push(...queryPool(ds, pack.item.filter, idx));
  }
  const unique = [...new Set(ids)];
  packCache.set(packId, unique);
  return unique;
}
