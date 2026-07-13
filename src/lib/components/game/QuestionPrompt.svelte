<script lang="ts">
	import type { QuestionView } from '$lib/bindings/Protocol';
	import { m } from '$lib/paraglide/messages';

	let { question }: { question: QuestionView } = $props();

	const variant = $derived(question.variant);
</script>

<!-- ponytail: prompt/media rendering is text-only; media (image/video/audio/
	youtube) is its own task — add when a question with media enters play. -->
<div class="prompt">
	<h2>{question.prompt.text}</h2>

	{#if variant.kind === 'MultipleChoice'}
		<ul class="choices">
			{#each variant.choices as choice (choice.id)}
				<li class="choice">{choice.text}</li>
			{/each}
		</ul>
	{:else if variant.kind === 'Order'}
		<ol class="order">
			{#each variant.items as item (item.id)}
				<li class="order-item">{item.text}</li>
			{/each}
		</ol>
	{:else if variant.kind === 'TrueFalse'}
		<p class="hint">{m.answer_true()} / {m.answer_false()}</p>
	{:else if variant.kind === 'Range'}
		<p class="hint">{variant.min} – {variant.max}</p>
	{/if}
	<!-- Open / NumericInput: prompt only, answered out loud -->
</div>

<style>
	.prompt {
		text-align: center;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-4);
	}
	.prompt h2 {
		font-family: var(--font-heading);
		margin: 0;
	}
	.choices,
	.order {
		list-style: none;
		margin: 0;
		padding: 0;
		width: min(28rem, 100%);
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
		counter-reset: item;
	}
	.choice,
	.order-item {
		padding: var(--space-3) var(--space-4);
		border: var(--border-width) var(--border-style) var(--border-color);
		border-radius: var(--radius-md);
		background: var(--bg-surface);
		text-align: left;
	}
	.order-item {
		counter-increment: item;
	}
	.order-item::before {
		content: counter(item) '. ';
		font-family: var(--font-heading);
		font-weight: 700;
		color: var(--color-text-muted);
	}
	.hint {
		color: var(--color-text-muted);
		margin: 0;
	}
</style>
