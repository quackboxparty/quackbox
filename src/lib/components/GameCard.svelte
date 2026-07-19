<script lang="ts">
	import type { Game } from '$lib/bindings/Games';
	import Card from './Card.svelte';

	let {
		game,
		selected = false,
		onclick
	}: { game: Game; selected?: boolean; onclick?: (e: MouseEvent) => void } = $props();
</script>

<Card {selected} {onclick}>
	<div class="body">
		<h3 class="title">{game.title}</h3>
		<p class="desc">{game.description}</p>
		{#if game.modes.length}
			<!-- <ul class="modes"> -->
			<!-- 	{#each game.modes as mode (mode)} -->
			<!-- 		<li class="badge">{mode}</li> -->
			<!-- 	{/each} -->
			<!-- </ul> -->
		{/if}
	</div>
</Card>

<style>
	/* layout on an element GameCard owns, so scoped styles apply */
	.body {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
		height: 100%;
	}
	.title {
		font-family: var(--font-heading);
		font-size: calc(1.125rem * var(--font-scale));
		color: var(--color-text);
	}
	.desc {
		color: var(--color-text-muted);
		font-size: calc(0.875rem * var(--font-scale));
		/* push mode badges toward the bottom for equal-height rows */
		flex: 1;
	}
	.modes {
		list-style: none;
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
		margin-top: var(--space-2);
		padding: 0;
	}
	.badge {
		font-family: var(--font-mono);
		font-size: calc(0.75rem * var(--font-scale));
		color: var(--color-text-muted);
		background: var(--bg-primary);
		border: var(--border-width) var(--border-style) var(--border-color);
		border-radius: var(--radius-full);
		padding: var(--space-1) var(--space-3);
		text-transform: capitalize;
	}
</style>
