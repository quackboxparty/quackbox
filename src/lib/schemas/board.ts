import * as v from 'valibot';

import { BoardId, LocalId, PackId, QuestionId, TagRef } from './common.ts';
import { PackFilter } from './pack.ts';

/** Point values on a board column — e.g. [100, 200, 300, 500]. */
const PointValues = v.pipe(v.array(v.pipe(v.number(), v.integer(), v.minValue(1))), v.minLength(2));

/** Mapping from point value → tags, used to narrow pool builds by difficulty. */
const DifficultyMap = v.record(v.string(), v.array(TagRef));

/**
 * A single board category. Resolution per `(point)` slot:
 * 1. explicit `question_ids[point]` — hard override, never overridden
 * 2. `pack_ref` — resolved pack's questions, AND-ed with filter/difficulty
 * 3. `filter` — dynamic pool pull, AND-ed with difficulty
 */
export const BoardCategory = v.pipe(
	v.strictObject({
		filter: v.optional(PackFilter),
		name: v.string(),
		pack_ref: v.optional(PackId),
		question_ids: v.optional(v.record(LocalId, QuestionId))
	}),
	v.check(
		(c) => Boolean(c.question_ids ?? c.pack_ref ?? c.filter),
		'each category must define at least one of: question_ids, pack_ref, filter'
	)
);

export const BoardFile = v.pipe(
	v.strictObject({
		categories: v.pipe(v.array(BoardCategory), v.minLength(2)),
		description: v.optional(v.string()),
		difficulty_map: v.optional(DifficultyMap),
		id: BoardId,
		points: PointValues,
		title: v.string()
	}),
	v.check((b) => {
		const dm = b.difficulty_map;
		if (dm === undefined) return true;
		return b.points.every((p) => dm[String(p)] !== undefined);
	}, 'difficulty_map must have an entry for every point in points, or be omitted entirely')
);

export type BoardCategory = v.InferOutput<typeof BoardCategory>;
export type BoardFile = v.InferOutput<typeof BoardFile>;
