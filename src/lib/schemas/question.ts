import { Effect, Schema } from 'effect';

import { Deprecation, License, LocaleCode, LocalId, QuestionId, Source, TagRef } from './common.ts';
import { MediaList } from './media.ts';

export const NormalizeOp = Schema.Literals([
	'lowercase',
	'strip_diacritics',
	'strip_punctuation',
	'strip_whitespace',
	'strip_articles'
]);
export type NormalizeOp = typeof NormalizeOp.Type;

const Choice = Schema.Struct({
	correct: Schema.optionalKey(Schema.Literal(true)),
	id: LocalId,
	media: Schema.optionalKey(MediaList),
	text: Schema.String
});
export type Choice = typeof Choice.Type;

const ChoicesList = Schema.Array(Choice)
	.check(
		Schema.isMinLength(2),
		Schema.makeFilter((choices) =>
			choices.some((c) => c.correct === true)
				? undefined
				: 'multiple_choice requires at least one choice with `correct: true`'
		),
		Schema.makeFilter((choices) => {
			const ids = new Set<string>();
			for (const c of choices) {
				if (ids.has(c.id)) return `duplicate choice id: ${c.id}`;
				ids.add(c.id);
			}
			return undefined;
		})
	)
	.annotate({ identifier: 'ChoicesList' });

const MultipleChoiceVariant = Schema.Struct({ choices: ChoicesList });

const TrueFalseVariant = Schema.Struct({});

const OpenVariant = Schema.Struct({
	accepted: Schema.Array(Schema.String).check(Schema.isMinLength(1)),
	normalize: Schema.optionalKey(Schema.Array(NormalizeOp))
});

const NumericInputVariant = Schema.Struct({
	tolerance: Schema.Number.check(Schema.isGreaterThanOrEqualTo(0)).pipe(
		Schema.optionalKey,
		Schema.withDecodingDefaultTypeKey(Effect.succeed(0))
	)
});

const RangeVariant = Schema.Struct({
	max: Schema.Number,
	min: Schema.Number,
	step: Schema.Number.check(Schema.isGreaterThanOrEqualTo(0)).pipe(
		Schema.optionalKey,
		Schema.withDecodingDefaultTypeKey(Effect.succeed(1))
	)
}).check(
	Schema.makeFilter((r) => (r.max > r.min ? undefined : 'range.max must be greater than range.min'))
);

const Prompt = Schema.Struct({
	media: Schema.optionalKey(MediaList),
	text: Schema.String
});

const TextVariants = Schema.Struct({
	multiple_choice: Schema.optionalKey(MultipleChoiceVariant),
	open: Schema.optionalKey(OpenVariant),
	true_false: Schema.optionalKey(TrueFalseVariant)
});

const NumericVariants = Schema.Struct({
	multiple_choice: Schema.optionalKey(MultipleChoiceVariant),
	numeric_input: Schema.optionalKey(NumericInputVariant),
	range: Schema.optionalKey(RangeVariant)
});

const TextContent = Schema.Struct({
	answer: Schema.String,
	default_lang: LocaleCode,
	explanation: Schema.optionalKey(Schema.String),
	prompt: Prompt,
	variants: TextVariants.check(
		Schema.makeFilter((vs) =>
			(vs.multiple_choice ?? vs.true_false ?? vs.open)
				? undefined
				: 'text question must define at least one variant'
		)
	)
});

const NumericContent = Schema.Struct({
	answer: Schema.Number,
	default_lang: LocaleCode,
	explanation: Schema.optionalKey(Schema.String),
	prompt: Prompt,
	unit: Schema.optionalKey(Schema.String),
	variants: NumericVariants.check(
		Schema.makeFilter((vs) =>
			(vs.multiple_choice ?? vs.numeric_input ?? vs.range)
				? undefined
				: 'numeric question must define at least one variant'
		)
	)
});

const OrderItem = Schema.Struct({
	id: LocalId,
	media: Schema.optionalKey(MediaList),
	position: Schema.Number.check(Schema.isInt(), Schema.isGreaterThanOrEqualTo(1)),
	text: Schema.String
});

const OrderContent = Schema.Struct({
	default_lang: LocaleCode,
	explanation: Schema.optionalKey(Schema.String),
	items: Schema.Array(OrderItem).check(
		Schema.isMinLength(2),
		Schema.makeFilter((arr) => {
			const ids = new Set<string>();
			for (const it of arr) {
				if (ids.has(it.id)) return `duplicate order item id: ${it.id}`;
				ids.add(it.id);
			}
			return undefined;
		}),
		Schema.makeFilter((arr) => {
			// positions must be a contiguous 1..N permutation
			const positions = arr.map((i) => i.position).sort((a, b) => a - b);
			return positions.every((p, idx) => p === idx + 1)
				? undefined
				: 'order items must have contiguous positions starting at 1, no duplicates';
		})
	),
	prompt: Prompt
});

const QuestionBase = {
	deprecated: Schema.optionalKey(Deprecation),
	id: QuestionId,
	lang_locked: Schema.optionalKey(LocaleCode),
	license: Schema.optionalKey(License),
	sources: Schema.optionalKey(Schema.Array(Source)),
	tags: Schema.Array(TagRef)
};

export const TextQuestion = Schema.Struct({
	...QuestionBase,
	content: TextContent,
	kind: Schema.Literal('text')
});

export const NumericQuestion = Schema.Struct({
	...QuestionBase,
	content: NumericContent,
	kind: Schema.Literal('numeric')
});

export const OrderQuestion = Schema.Struct({
	...QuestionBase,
	content: OrderContent,
	kind: Schema.Literal('order')
});

export const Question = Schema.Union([TextQuestion, NumericQuestion, OrderQuestion]);
export type Question = typeof Question.Type;

export const QuestionFile = Schema.Array(Question);
export type QuestionFile = typeof QuestionFile.Type;

export const QuestionKind = Schema.Literals(['text', 'numeric', 'order']);
export type QuestionKind = typeof QuestionKind.Type;

export const VariantName = Schema.Literals([
	'multiple_choice',
	'true_false',
	'open',
	'numeric_input',
	'range'
]);
export type VariantName = typeof VariantName.Type;
