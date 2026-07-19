<script lang="ts">
	import type { ClientView } from '$lib/bindings/Protocol';
	import { m } from '$lib/paraglide/messages';
	import Lobby from './Lobby.svelte';
	import BoardSelect from './BoardSelect.svelte';
	import QuestionOpen from './QuestionOpen.svelte';
	import Reveal from './Reveal.svelte';
	import GameOver from './GameOver.svelte';

	let { view }: { view: ClientView } = $props();

	const grid = $derived(view.stage.kind === 'GridQuiz' ? view.stage : null);
</script>

<article class="stage">
	{#if grid}
		{#if (grid.phase === 'question_open' || grid.phase === 'reveal') &&
		grid.current_category !== null &&
		grid.current_points !== null}
			<p class="question-context">
				{m.question_cell({ category: grid.current_category, points: grid.current_points })}
			</p>
		{/if}

		{#if grid.phase === 'lobby'}
			<Lobby players={view.players} />
		{:else if grid.phase === 'board_select'}
			<BoardSelect view={grid} />
		{:else if grid.phase === 'question_open'}
			<QuestionOpen view={grid} question={view.question} />
		{:else if grid.phase === 'reveal'}
			<Reveal question={view.question} players={view.players} judgmentLog={view.judgment_log} />
		{:else if grid.phase === 'game_over'}
			<GameOver players={view.players} />
		{:else}
			{grid.phase satisfies never}
		{/if}
	{:else}
		<p class="fallback">{m.mode_not_supported()}</p>
	{/if}
</article>

<style>
	.stage {
		min-height: 100%;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-6);
		padding: var(--space-8) var(--space-4);
		min-width: 0;
		max-width: 100%;
		overflow-x: hidden;
		container-type: inline-size;
	}
	.fallback {
		color: var(--color-text-muted);
		text-align: center;
		padding: var(--space-12) var(--space-4);
	}
	.question-context {
		margin: 0;
		color: var(--color-primary);
		font-family: var(--font-heading);
		font-size: calc(1.1rem * var(--font-scale));
		font-weight: 700;
	}
</style>
