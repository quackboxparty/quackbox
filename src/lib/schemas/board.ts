import { Schema } from 'effect';

import { BoardId, LocalId, PackId, QuestionId, TagRef } from './common.ts';
import { PackFilter } from './pack.ts';

/** Point values on a board column — e.g. [100, 200, 300, 500]. */
const PointValues = Schema.Array(
	Schema.Number.check(Schema.isInt(), Schema.isGreaterThanOrEqualTo(1))
).check(Schema.isMinLength(2));

/** Mapping from point value → tags, used to narrow pool builds by difficulty. */
const DifficultyMap = Schema.Record(Schema.String, Schema.Array(TagRef));

/**
 * A single board category. Resolution per `(point)` slot:
 * 1. explicit `question_ids[point]` — hard override, never overridden
 * 2. `pack_ref` — resolved pack's questions, AND-ed with filter/difficulty
 * 3. `filter` — dynamic pool pull, AND-ed with difficulty
 */
export const BoardCategory = Schema.Struct({
	filter: Schema.optionalKey(PackFilter),
	name: Schema.String,
	pack_ref: Schema.optionalKey(PackId),
	question_ids: Schema.optionalKey(Schema.Record(LocalId, QuestionId))
}).check(
	Schema.makeFilter((c) =>
		(c.question_ids ?? c.pack_ref ?? c.filter)
			? undefined
			: 'each category must define at least one of: question_ids, pack_ref, filter'
	)
);

export const BoardFile = Schema.Struct({
	categories: Schema.Array(BoardCategory).check(Schema.isMinLength(2)),
	description: Schema.optionalKey(Schema.String),
	difficulty_map: Schema.optionalKey(DifficultyMap),
	id: BoardId,
	points: PointValues,
	title: Schema.String
}).check(
	Schema.makeFilter((b) => {
		const dm = b.difficulty_map;
		if (dm === undefined) return undefined;
		return b.points.every((p) => dm[String(p)] !== undefined)
			? undefined
			: 'difficulty_map must have an entry for every point in points, or be omitted entirely';
	})
);

export type BoardCategory = typeof BoardCategory.Type;
export type BoardFile = typeof BoardFile.Type;
