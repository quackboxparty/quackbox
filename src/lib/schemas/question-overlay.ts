import * as v from 'valibot';
import { QuestionId } from './common.ts';
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
 */

const LocalId = v.pipe(v.string(), v.regex(/^[a-z0-9][a-z0-9_]*$/));

const ChoiceOverlay = v.strictObject({
	id: LocalId,
	text: v.optional(v.string()),
	media: v.optional(MediaList)
});

const MultipleChoiceVariantOverlay = v.strictObject({
	choices: v.optional(v.array(ChoiceOverlay))
});

const OpenVariantOverlay = v.strictObject({
	accepted: v.optional(v.array(v.string()))
});

const PromptOverlay = v.strictObject({
	text: v.optional(v.string()),
	media: v.optional(MediaList)
});

const TextVariantsOverlay = v.strictObject({
	multiple_choice: v.optional(MultipleChoiceVariantOverlay),
	open: v.optional(OpenVariantOverlay)
	// true_false has no translatable fields
});

const NumericVariantsOverlay = v.strictObject({
	multiple_choice: v.optional(MultipleChoiceVariantOverlay)
	// numeric_input / range have nothing translatable
});

const TextContentOverlay = v.strictObject({
	prompt: v.optional(PromptOverlay),
	answer: v.optional(v.string()),
	explanation: v.optional(v.string()),
	variants: v.optional(TextVariantsOverlay)
});

const NumericContentOverlay = v.strictObject({
	prompt: v.optional(PromptOverlay),
	unit: v.optional(v.string()),
	explanation: v.optional(v.string()),
	variants: v.optional(NumericVariantsOverlay)
});

const OrderItemOverlay = v.strictObject({
	id: LocalId,
	text: v.optional(v.string()),
	media: v.optional(MediaList)
});

const OrderContentOverlay = v.strictObject({
	prompt: v.optional(PromptOverlay),
	explanation: v.optional(v.string()),
	items: v.optional(v.array(OrderItemOverlay))
});

/*
 * Overlay files don't carry `kind` — they're keyed only by `id` and patch
 * whichever fields are present. Loader resolves the kind from the canonical
 * question and applies the matching shape. We accept the union of all three
 * content shapes; strict objects keep the boundaries clean per kind.
 */
const ContentOverlay = v.union([TextContentOverlay, NumericContentOverlay, OrderContentOverlay]);

export const QuestionOverlay = v.strictObject({
	id: QuestionId,
	content: ContentOverlay
});
export type QuestionOverlay = v.InferOutput<typeof QuestionOverlay>;

export const QuestionOverlayFile = v.array(QuestionOverlay);
export type QuestionOverlayFile = v.InferOutput<typeof QuestionOverlayFile>;
