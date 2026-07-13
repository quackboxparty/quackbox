<script lang="ts">
	import type { PlayerView } from '$lib/bindings/Protocol';
	import { playerColor, playerInitial, sortedByScore } from '$lib/playerUi';
	import { room } from '$lib/room.svelte';
	import { m } from '$lib/paraglide/messages';

	let { players }: { players: Record<string, PlayerView> } = $props();

	const standings = $derived(sortedByScore(players));
	const winner = $derived(standings[0]?.[0] ?? null);
</script>

<!-- ponytail: display-only. Play-again / next-game is a mod control for a later
	todo (chaining exists, #9), not a phase component. -->
<section class="over">
	<header class="head">
		<h1>{m.game_over()}</h1>
		{#if winner}<p class="muted">🏆 {m.winner_wins({ name: winner })}</p>{/if}
	</header>

	<ol class="standings">
		{#each standings as [name, p], i (name)}
			<li class="row" class:you={name === room.player} class:dim={!p.connected}>
				<span class="rank">{i + 1}</span>
				<span class="avatar" style:background={playerColor(name)}>{playerInitial(name)}</span>
				<span class="name">{name}</span>
				<span class="score">{p.score}</span>
			</li>
		{/each}
	</ol>
</section>

<style>
	.over {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-6);
	}
	.head {
		text-align: center;
	}
	.head h1 {
		font-family: var(--font-heading);
		margin: 0;
	}
	.muted {
		color: var(--color-text-muted);
		font-size: calc(0.9rem * var(--font-scale));
	}
	.standings {
		list-style: none;
		margin: 0;
		padding: 0;
		width: min(24rem, 100%);
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}
	.row {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-2) var(--space-3);
		border: var(--border-width) var(--border-style) var(--border-color);
		border-radius: var(--radius-md);
		background: var(--bg-surface);
	}
	.row.you {
		border-color: var(--color-primary);
	}
	.dim {
		opacity: 0.4;
	}
	.rank {
		font-family: var(--font-heading);
		font-weight: 700;
		color: var(--color-text-muted);
		min-width: 1.5rem;
	}
	.avatar {
		width: 1.75rem;
		height: 1.75rem;
		border-radius: var(--radius-full);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--color-text-inverse);
		font-weight: 700;
		font-family: var(--font-heading);
		font-size: calc(0.8rem * var(--font-scale));
		flex-shrink: 0;
		text-transform: uppercase;
	}
	.name {
		flex: 1;
	}
	.score {
		font-family: var(--font-heading);
		font-weight: 700;
	}
</style>
