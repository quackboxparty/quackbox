import * as v from 'valibot';

import { GamemodeId } from './common.ts';
import { QuestionKind, VariantName } from './question.ts';

const Accepts = v.pipe(
	v.strictObject({
		kinds: v.pipe(v.array(QuestionKind), v.minLength(1)),
		max_choices: v.optional(v.pipe(v.number(), v.integer(), v.minValue(2))),
		min_choices: v.optional(v.pipe(v.number(), v.integer(), v.minValue(2))),
		variants: v.pipe(v.array(VariantName), v.minLength(1))
	}),
	v.check(
		(a) =>
			a.min_choices === undefined || a.max_choices === undefined || a.max_choices >= a.min_choices,
		'accepts.max_choices must be >= min_choices'
	)
);

const Requires = v.strictObject({
	max_players: v.optional(v.pipe(v.number(), v.integer(), v.minValue(1))),
	min_players: v.optional(v.pipe(v.number(), v.integer(), v.minValue(1))),
	timer: v.optional(v.boolean())
});

// Svelte component file names; loader resolves them inside the gamemode dir.
const UiFile = v.pipe(v.string(), v.regex(/^[A-Za-z0-9_.-]+\.svelte$/));

const Ui = v.strictObject({
	host_view: UiFile,
	player_view: UiFile,
	spectator_view: v.optional(UiFile)
});

export const GamemodeManifest = v.strictObject({
	accepts: Accepts,
	description: v.optional(v.string()),
	id: GamemodeId,
	name: v.string(),
	requires: v.optional(Requires),
	ui: Ui
});
export type GamemodeManifest = v.InferOutput<typeof GamemodeManifest>;
