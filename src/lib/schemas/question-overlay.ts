import * as v from 'valibot';

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
 * Strict objects enforce all of the above by rejecting unknown keys.
 *
 * Overlay files don't carry `kind` — they're keyed only by `id` and patch
 * whichever fields are present. Loader resolves the kind from the canonical
 * question and applies the matching shape. A single catch-all strict object
 * (all fields optional) avoids valibot disambiguation issues with all-optional
 * union branches. Kind-specific field mismatches (e.g. `unit` on a text
 * question) are silently ignored at merge-time.
 */

const ChoiceOverlay = v.strictObject({
	id: LocalId,
	media: v.optional(MediaList),
	text: v.optional(v.string())
});

const PromptOverlay = v.strictObject({
	media: v.optional(MediaList),
	text: v.optional(v.string())
});

const OrderItemOverlay = v.strictObject({
	id: LocalId,
	media: v.optional(MediaList),
	text: v.optional(v.string())
});

/**
 * Single translatable-shape overlay covering all three question kinds.
 * All fields optional — only present fields are applied by the loader.
 * Non-translatable fields (`correct`, `position`, numeric `answer`,
 * `tolerance`, `min`/`max`/`step`, `normalize`, `default_lang`) are
 * deliberately absent to reject them at schema level.
 */
const ContentOverlay = v.strictObject({
	answer: v.optional(v.string()),
	explanation: v.optional(v.string()),
	items: v.optional(v.array(OrderItemOverlay)),
	prompt: v.optional(PromptOverlay),
	unit: v.optional(v.string()),
	variants: v.optional(
		v.strictObject({
			multiple_choice: v.optional(
				v.strictObject({
					choices: v.optional(v.array(ChoiceOverlay))
				})
			),
			open: v.optional(
				v.strictObject({
					accepted: v.optional(v.array(v.string()))
				})
			)
			// true_false, numeric_input, range have no translatable fields
		})
	)
});

export const QuestionOverlay = v.strictObject({
	content: ContentOverlay,
	id: QuestionId
});
export type QuestionOverlay = v.InferOutput<typeof QuestionOverlay>;

export const QuestionOverlayFile = v.array(QuestionOverlay);
export type QuestionOverlayFile = v.InferOutput<typeof QuestionOverlayFile>;
