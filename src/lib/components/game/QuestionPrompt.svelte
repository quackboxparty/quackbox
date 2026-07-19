<script lang="ts">
	import type { QuestionView } from '$lib/bindings/Protocol';
	import { m } from '$lib/paraglide/messages';
	import MediaDisplay from './MediaDisplay.svelte';

	let { question }: { question: QuestionView } = $props();

	const variant = $derived(question.variant);
</script>

<div class="prompt">
	<h2>{question.prompt.text}</h2>

	{#if question.prompt.media}
		<MediaDisplay media={question.prompt.media} />
	{/if}

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
		width: min(60rem, 100%);
		padding: var(--space-8);
		background: var(--bg-surface-elevated);
		border: var(--border-width) var(--border-style) var(--border-color);
		border-radius: var(--radius-lg);
		box-shadow: var(--shadow-md);
	}
	.prompt h2 {
		font-family: var(--font-heading);
		margin: 0;
		/* scale with viewport: 1.4rem on phones, 3.5rem on wide screens */
		font-size: clamp(1.4rem, 6cqi, 3.5rem);
		line-height: 1.15;
	}
	.choices,
	.order {
		list-style: none;
		margin: 0;
		padding: 0;
		width: min(50rem, 100%);
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		counter-reset: item;
	}
	.choice,
	.order-item {
		padding: var(--space-4) var(--space-6);
		border: var(--border-width) var(--border-style) var(--border-color);
		border-radius: var(--radius-md);
		background: var(--bg-surface);
		text-align: left;
		font-size: clamp(1rem, 3.2cqi, 1.6rem);
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
		font-size: clamp(1.1rem, 3cqi, 1.6rem);
	}
</style>
