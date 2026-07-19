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
		width: 100%;
	}
	.head {
		text-align: center;
	}
	.head h1 {
		font-family: var(--font-heading);
		margin: 0;
		font-size: clamp(2rem, 7cqi, 4rem);
	}
	.muted {
		color: var(--color-text-muted);
		font-size: clamp(0.95rem, 2.2cqi, 1.4rem);
	}
	.standings {
		list-style: none;
		margin: 0;
		padding: 0;
		width: min(40rem, 100%);
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}
	.row {
		display: flex;
		align-items: center;
		gap: var(--space-4);
		padding: var(--space-3) var(--space-4);
		border: var(--border-width) var(--border-style) var(--border-color);
		border-radius: var(--radius-md);
		background: var(--bg-surface);
		font-size: clamp(1rem, 2.6cqi, 1.5rem);
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
		min-width: 1.8rem;
	}
	.avatar {
		width: clamp(1.75rem, 5cqi, 2.75rem);
		height: clamp(1.75rem, 5cqi, 2.75rem);
		border-radius: var(--radius-full);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--color-text-inverse);
		font-weight: 700;
		font-family: var(--font-heading);
		font-size: clamp(0.8rem, 2.2cqi, 1.2rem);
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
