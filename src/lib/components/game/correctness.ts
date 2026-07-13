import type { AnswerView, CorrectnessView, VariantView } from '$lib/bindings/Protocol';
import { m } from '$lib/paraglide/messages';

/**
 * Human-readable correct answer for any correctness kind. MC/Order carry only
 * ids on the correctness side, so the variant is needed to resolve id -> text.
 * Exhaustive over CorrectnessView: a new kind is a compile error at the `never`.
 */
export function correctnessText(answer: AnswerView, variant: VariantView): string {
	const c: CorrectnessView = answer.correctness;
	switch (c.kind) {
		case 'MultipleChoice': {
			const choices = variant.kind === 'MultipleChoice' ? variant.choices : [];
			const ids = new Set(c.correct_ids);
			return choices
				.filter((ch) => ids.has(ch.id))
				.map((ch) => ch.text)
				.join(', ');
		}
		case 'Open':
			return c.accepted.join(', ');
		case 'TrueFalse':
			return c.correct ? m.answer_true() : m.answer_false();
		case 'Numeric':
			return c.tolerance > 0 ? `${c.value} ± ${c.tolerance}` : String(c.value);
		case 'Order': {
			const items = variant.kind === 'Order' ? variant.items : [];
			const text = new Map(items.map((it) => [it.id, it.text]));
			return [...c.positions]
				.sort((a, b) => a.position - b.position)
				.map((p) => text.get(p.id) ?? p.id)
				.join(' → ');
		}
		default:
			return c satisfies never;
	}
}
