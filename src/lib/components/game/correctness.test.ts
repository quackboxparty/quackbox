import { describe, it, expect, vi } from 'vitest';

vi.mock('$lib/paraglide/messages', () => ({
	m: { answer_true: () => 'True', answer_false: () => 'False' }
}));

import { correctnessText } from './correctness';

describe('correctnessText', () => {
	it('MultipleChoice resolves correct ids to choice text', () => {
		expect(
			correctnessText(
				{ correctness: { kind: 'MultipleChoice', correct_ids: ['b', 'c'] }, explanation: null },
				{
					kind: 'MultipleChoice',
					choices: [
						{ id: 'a', text: 'A', media: null },
						{ id: 'b', text: 'B', media: null },
						{ id: 'c', text: 'C', media: null }
					]
				}
			)
		).toBe('B, C');
	});

	it('Open joins accepted answers', () => {
		expect(
			correctnessText(
				{ correctness: { kind: 'Open', accepted: ['Paris', 'Lutetia'] }, explanation: null },
				{ kind: 'Open' }
			)
		).toBe('Paris, Lutetia');
	});

	it('TrueFalse maps the boolean', () => {
		expect(
			correctnessText(
				{ correctness: { kind: 'TrueFalse', correct: true }, explanation: null },
				{ kind: 'TrueFalse' }
			)
		).toBe('True');
	});

	it('Numeric shows tolerance only when positive', () => {
		expect(
			correctnessText(
				{ correctness: { kind: 'Numeric', value: 42, tolerance: 0 }, explanation: null },
				{ kind: 'NumericInput' }
			)
		).toBe('42');
		expect(
			correctnessText(
				{ correctness: { kind: 'Numeric', value: 42, tolerance: 2 }, explanation: null },
				{ kind: 'NumericInput' }
			)
		).toBe('42 ± 2');
	});

	it('Order sorts by position and resolves item text', () => {
		expect(
			correctnessText(
				{
					correctness: {
						kind: 'Order',
						positions: [
							{ id: 'y', position: 1 },
							{ id: 'x', position: 0 }
						]
					},
					explanation: null
				},
				{
					kind: 'Order',
					items: [
						{ id: 'x', text: 'First', media: null },
						{ id: 'y', text: 'Second', media: null }
					]
				}
			)
		).toBe('First → Second');
	});
});
