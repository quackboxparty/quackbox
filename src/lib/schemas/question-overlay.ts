import { Schema } from 'effect';

import { LocalId, QuestionId } from './common.ts';
import { MediaList } from './media.ts';

/*
 * Overlay = translatable subset of canonical question.
 *
 * Hard rules (data-model.md §Translation overlay):
 * - No `correct` on choices.
 * - No `position` on order items.
 * - No `answer` on numeric questions (answer is the fact itself).
 * - No variant config: `tolerance`, `min`, `max`, `step`, `normalize`.
 * - No `default_lang`, no tags/license/sources/lang_locked/deprecated.
 *
 * Schema.Struct rejects unknown keys (strict by default in v4).
 *
 * Overlay files don't carry `kind` — they're keyed only by `id` and patch
 * whichever fields are present. Loader resolves the kind from the canonical
 * question and applies the matching shape. A single catch-all struct
 * (all fields optional) avoids disambiguation issues with all-optional union
 * branches. Kind-specific field mismatches (e.g. `unit` on a text question)
 * are silently ignored at merge-time.
 */

const ChoiceOverlay = Schema.Struct({
	id: LocalId,
	media: Schema.optionalKey(MediaList),
	text: Schema.optionalKey(Schema.String)
});

const PromptOverlay = Schema.Struct({
	media: Schema.optionalKey(MediaList),
	text: Schema.optionalKey(Schema.String)
});

const OrderItemOverlay = Schema.Struct({
	id: LocalId,
	media: Schema.optionalKey(MediaList),
	text: Schema.optionalKey(Schema.String)
});

/**
 * Single translatable-shape overlay covering all three question kinds.
 * All fields optional — only present fields are applied by the loader.
 * Non-translatable fields (`correct`, `position`, numeric `answer`,
 * `tolerance`, `min`/`max`/`step`, `normalize`, `default_lang`) are
 * deliberately absent to reject them at schema level.
 */
const ContentOverlay = Schema.Struct({
	answer: Schema.optionalKey(Schema.String),
	explanation: Schema.optionalKey(Schema.String),
	items: Schema.optionalKey(Schema.Array(OrderItemOverlay)),
	prompt: Schema.optionalKey(PromptOverlay),
	unit: Schema.optionalKey(Schema.String),
	variants: Schema.optionalKey(
		Schema.Struct({
			multiple_choice: Schema.optionalKey(
				Schema.Struct({
					choices: Schema.optionalKey(Schema.Array(ChoiceOverlay))
				})
			),
			open: Schema.optionalKey(
				Schema.Struct({
					accepted: Schema.optionalKey(Schema.Array(Schema.String))
				})
			)
			// true_false, numeric_input, range have no translatable fields
		})
	)
});

export const QuestionOverlay = Schema.Struct({
	content: ContentOverlay,
	id: QuestionId
});
export type QuestionOverlay = typeof QuestionOverlay.Type;

export const QuestionOverlayFile = Schema.Array(QuestionOverlay);
export type QuestionOverlayFile = typeof QuestionOverlayFile.Type;
