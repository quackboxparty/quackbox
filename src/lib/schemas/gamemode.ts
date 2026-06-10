import { Schema } from 'effect';

import { GamemodeId } from './common.ts';
import { QuestionKind, VariantName } from './question.ts';

const Accepts = Schema.Struct({
	kinds: Schema.Array(QuestionKind).check(Schema.isMinLength(1)),
	max_choices: Schema.optionalKey(
		Schema.Number.check(Schema.isInt(), Schema.isGreaterThanOrEqualTo(2))
	),
	min_choices: Schema.optionalKey(
		Schema.Number.check(Schema.isInt(), Schema.isGreaterThanOrEqualTo(2))
	),
	variants: Schema.Array(VariantName).check(Schema.isMinLength(1))
}).check(
	Schema.makeFilter((a) =>
		a.min_choices === undefined || a.max_choices === undefined || a.max_choices >= a.min_choices
			? undefined
			: 'accepts.max_choices must be >= min_choices'
	)
);

const Requires = Schema.Struct({
	max_players: Schema.optionalKey(
		Schema.Number.check(Schema.isInt(), Schema.isGreaterThanOrEqualTo(1))
	),
	min_players: Schema.optionalKey(
		Schema.Number.check(Schema.isInt(), Schema.isGreaterThanOrEqualTo(1))
	),
	timer: Schema.optionalKey(Schema.Boolean)
});

// Svelte component file names; loader resolves them inside the gamemode dir.
const UiFile = Schema.String.check(Schema.isPattern(/^[A-Za-z0-9_.-]+\.svelte$/));

const Ui = Schema.Struct({
	host_view: UiFile,
	player_view: UiFile,
	spectator_view: Schema.optionalKey(UiFile)
});

export const GamemodeManifest = Schema.Struct({
	accepts: Accepts,
	description: Schema.optionalKey(Schema.String),
	id: GamemodeId,
	name: Schema.String,
	requires: Schema.optionalKey(Requires),
	ui: Ui
});
export type GamemodeManifest = typeof GamemodeManifest.Type;
