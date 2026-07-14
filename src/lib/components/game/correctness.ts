import type { AnswerView, CorrectnessView, VariantView } from '$lib/bindings/Protocol';
import { m } from '$lib/paraglide/messages';

/**
 * Human-readable correct answer for any correctness kind. MC/Order carry only
 * ids on the correctness side, so the variant is needed to resolve id -> text.
 * Exhaustive over CorrectnessView: a new kind is a compile error at the `never`.
 */
export function correctnessText(answer: AnswerView, variant: VariantView): string {
  const correct: CorrectnessView = answer.correctness;
  switch (correct.kind) {
    case 'MultipleChoice': {
      const choices = variant.kind === 'MultipleChoice' ? variant.choices : [];
      const ids = new Set(correct.correct_ids);
      return choices
        .filter((choice) => ids.has(choice.id))
        .map((choice) => choice.text)
        .join(', ');
    }
    case 'Open':
      return correct.accepted.join(', ');
    case 'TrueFalse':
      return correct.correct ? m.answer_true() : m.answer_false();
    case 'Numeric':
      return correct.tolerance > 0 ? `${correct.value} ± ${correct.tolerance}` : String(correct.value);
    case 'Order': {
      const items = variant.kind === 'Order' ? variant.items : [];
      const text = new Map(items.map((it) => [it.id, it.text]));
      return [...correct.positions]
        .sort((a, b) => a.position - b.position)
        .map((p) => text.get(p.id) ?? p.id)
        .join(' → ');
    }
    default:
      return correct satisfies never;
  }
}
