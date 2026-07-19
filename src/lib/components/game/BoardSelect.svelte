<script lang="ts">
	import type { GridQuizView } from '$lib/bindings/Protocol';
	import { playerColor } from '$lib/playerUi';
	import { room, has } from '$lib/room.svelte';
	import { m } from '$lib/paraglide/messages';

	let { view }: { view: GridQuizView } = $props();

	const activePicker = $derived(view.active_picker);
	const isMyTurn = $derived(activePicker !== null && activePicker === room.player);
	const canPick = $derived(isMyTurn || has('Moderate'));

	// Transient disable: which cell we just picked, cleared when a snapshot
	// moves us off board_select (the phase switch unmounts us) or re-renders.
	let pending = $state<{ column: number; row: number } | null>(null);
	$effect(() => {
		void view; // clear stale pending once a fresh snapshot lands
		pending = null;
	});

	function pick(category: number, point: number) {
		if (!canPick || pending) return;
		pending = { column: category, row: point };
		room.send?.({ kind: 'PickCell', category, point });
	}
</script>

<section class="board">
	<div class="banner">
		{#if isMyTurn}
			🎯 <strong>{m.your_turn_pick()}</strong>
		{:else if activePicker}
			🎯 <strong style:color={playerColor(activePicker)}
				>{m.player_choosing({ name: activePicker })}</strong
			>
		{/if}
	</div>

	<div class="grid" style:--cols={view.categories.length}>
		{#each view.categories as cat (cat)}
			<div class="col-head">{cat}</div>
		{/each}
		{#each view.points as _point, r (r)}
			{#each view.categories as _cat, c (c)}
				{@const used = view.used[c]?.[r] ?? false}
				{@const isPending = pending !== null && pending.column === c && pending.row === r}
				<button
					class="cell"
					class:used
					disabled={used || !canPick || isPending}
					class:pending={isPending}
					onclick={() => {
						pick(c, r);
					}}
				>
					{#if used}—{:else}{view.points[r]}{/if}
				</button>
			{/each}
		{/each}
	</div>
</section>

<style>
	.board {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-6);
		width: 100%;
		max-width: 100%;
		min-width: 0;
	}
	.banner {
		font-size: calc(1.1rem * var(--font-scale));
		padding: var(--space-3) var(--space-4);
		border-radius: var(--radius-md);
		background: var(--bg-surface);
		border: var(--border-width) var(--border-style) var(--border-color);
		max-width: 100%;
		min-width: 0;
		overflow-wrap: break-word;
		text-align: center;
	}
	.grid {
		display: grid;
		/* minmax(0, 1fr) lets columns shrink below min-content so the grid
		   never blows past the viewport on long category names or 6+ cats */
		grid-template-columns: repeat(var(--cols, 4), minmax(0, 1fr));
		gap: var(--space-2);
		/* grow on big screens so the board isn't a tiny island in a sea of
		   whitespace; caps at 80rem so 4K doesn't get comically huge */
		width: min(80rem, 100%);
		min-width: 0;
		container-type: inline-size;
	}
	.col-head {
		font-family: var(--font-heading);
		text-align: center;
		padding: var(--space-2);
		/* scale with column width so heads stay readable in narrow columns
		   and grow with the grid on wide screens */
		font-size: clamp(0.55rem, 3.2cqi, 1.6rem);
		line-height: 1.15;
		background: var(--color-primary);
		color: var(--color-text-inverse);
		border-radius: var(--radius-sm);
		min-width: 0;
		overflow: hidden;
		overflow-wrap: break-word;
		hyphens: auto;
	}
	.cell {
		font-family: var(--font-heading);
		/* scale with column width: bigger on wide grids, smaller on phones */
		font-size: clamp(0.95rem, 7.5cqi, 3.5rem);
		font-weight: 700;
		padding: var(--space-4) var(--space-2);
		background: var(--bg-surface-elevated);
		border: var(--border-width) var(--border-style) var(--border-color);
		border-radius: var(--radius-sm);
		color: var(--color-primary);
		cursor: pointer;
		min-width: 0;
		overflow: hidden;
		text-align: center;
		display: flex;
		align-items: center;
		justify-content: center;
	}
	@media (max-width: 480px) {
		.cell {
			padding: var(--space-3) var(--space-1);
		}
		.col-head {
			padding: var(--space-2) var(--space-1);
		}
	}
	.cell:disabled {
		cursor: default;
	}
	.cell.used {
		opacity: 0.25;
		color: var(--color-text-muted);
	}
	.cell.pending {
		opacity: 0.5;
	}
</style>
