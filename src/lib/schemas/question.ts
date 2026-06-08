import * as v from 'valibot';

import { Deprecation, License, LocaleCode, LocalId, QuestionId, Source, TagRef } from './common.ts';
import { MediaList } from './media.ts';

export const NormalizeOp = v.picklist([
	'lowercase',
	'strip_diacritics',
	'strip_punctuation',
	'strip_whitespace',
	'strip_articles'
]);
export type NormalizeOp = v.InferOutput<typeof NormalizeOp>;

const Choice = v.strictObject({
	correct: v.optional(v.literal(true)),
	id: LocalId,
	media: v.optional(MediaList),
	text: v.string()
});
export type Choice = v.InferOutput<typeof Choice>;

const ChoicesList = v.pipe(
	v.array(Choice),
	v.minLength(2),
	v.check(
		(choices) => choices.some((c) => c.correct === true),
		'multiple_choice requires at least one choice with `correct: true`'
	),
	v.check((choices) => {
		const ids = new Set<string>();
		for (const c of choices) {
			if (ids.has(c.id)) return false;
			ids.add(c.id);
		}
		return true;
	}, 'multiple_choice choice ids must be unique')
);

const MultipleChoiceVariant = v.strictObject({
	choices: ChoicesList
});

const TrueFalseVariant = v.strictObject({});

const OpenVariant = v.strictObject({
	accepted: v.pipe(v.array(v.string()), v.minLength(1)),
	normalize: v.optional(v.array(NormalizeOp))
});

const NumericInputVariant = v.strictObject({
	tolerance: v.optional(v.pipe(v.number(), v.minValue(0)), 0)
});

const RangeVariant = v.pipe(
	v.strictObject({
		max: v.number(),
		min: v.number(),
		step: v.optional(v.pipe(v.number(), v.minValue(0)), 1)
	}),
	v.check((r) => r.max > r.min, 'range.max must be greater than range.min')
);

const Prompt = v.strictObject({
	media: v.optional(MediaList),
	text: v.string()
});

const TextVariants = v.strictObject({
	multiple_choice: v.optional(MultipleChoiceVariant),
	open: v.optional(OpenVariant),
	true_false: v.optional(TrueFalseVariant)
});

const NumericVariants = v.strictObject({
	multiple_choice: v.optional(MultipleChoiceVariant),
	numeric_input: v.optional(NumericInputVariant),
	range: v.optional(RangeVariant)
});

const TextContent = v.strictObject({
	answer: v.string(),
	default_lang: LocaleCode,
	explanation: v.optional(v.string()),
	prompt: Prompt,
	variants: v.pipe(
		TextVariants,
		v.check(
			(vs) => Boolean(vs.multiple_choice ?? vs.true_false ?? vs.open),
			'text question must define at least one variant'
		)
	)
});

const NumericContent = v.strictObject({
	answer: v.number(),
	default_lang: LocaleCode,
	explanation: v.optional(v.string()),
	prompt: Prompt,
	unit: v.optional(v.string()),
	variants: v.pipe(
		NumericVariants,
		v.check(
			(vs) => Boolean(vs.multiple_choice ?? vs.numeric_input ?? vs.range),
			'numeric question must define at least one variant'
		)
	)
});

const OrderItem = v.strictObject({
	id: LocalId,
	media: v.optional(MediaList),
	position: v.pipe(v.number(), v.integer(), v.minValue(1)),
	text: v.string()
});

const OrderContent = v.pipe(
	v.strictObject({
		default_lang: LocaleCode,
		explanation: v.optional(v.string()),
		items: v.pipe(v.array(OrderItem), v.minLength(2)),
		prompt: Prompt
	}),
	v.check((c) => {
		const ids = new Set<string>();
		for (const it of c.items) {
			if (ids.has(it.id)) return false;
			ids.add(it.id);
		}
		return true;
	}, 'order items must have unique ids'),
	v.check((c) => {
		// positions must be a contiguous 1..N permutation
		const positions = c.items.map((i) => i.position).sort((a, b) => a - b);
		return positions.every((p, idx) => p === idx + 1);
	}, 'order items must have contiguous positions starting at 1, no duplicates')
);

const QuestionBase = {
	deprecated: v.optional(Deprecation),
	id: QuestionId,
	lang_locked: v.optional(LocaleCode),
	license: v.optional(License),
	sources: v.optional(v.array(Source)),
	tags: v.array(TagRef)
};

export const TextQuestion = v.strictObject({
	...QuestionBase,
	content: TextContent,
	kind: v.literal('text')
});

export const NumericQuestion = v.strictObject({
	...QuestionBase,
	content: NumericContent,
	kind: v.literal('numeric')
});

export const OrderQuestion = v.strictObject({
	...QuestionBase,
	content: OrderContent,
	kind: v.literal('order')
});

export const Question = v.variant('kind', [TextQuestion, NumericQuestion, OrderQuestion]);
export type Question = v.InferOutput<typeof Question>;

export const QuestionFile = v.array(Question);
export type QuestionFile = v.InferOutput<typeof QuestionFile>;

export const QuestionKind = v.picklist(['text', 'numeric', 'order']);
export type QuestionKind = v.InferOutput<typeof QuestionKind>;

export const VariantName = v.picklist([
	'multiple_choice',
	'true_false',
	'open',
	'numeric_input',
	'range'
]);
export type VariantName = v.InferOutput<typeof VariantName>;
