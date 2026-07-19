<script lang="ts">
	import type { PlayerView } from '$lib/bindings/Protocol';
	import { playerColor, playerInitial } from '$lib/playerUi';
	import { room, has } from '$lib/room.svelte';
	import { m } from '$lib/paraglide/messages';
	import Button from '$lib/components/Button.svelte';

	let { players }: { players: Record<string, PlayerView> } = $props();

	const player_entries = $derived(Object.entries(players));
	const can_start = $derived(
		player_entries.some(([, p]) => p.connected && p.grants.includes('Play'))
	);
</script>

<section class="lobby">
	<header class="head">
		<h1>{m.lobby()}</h1>
		<p class="muted">{m.players_joined({ count: player_entries.length })}</p>
	</header>

	<ul class="roster">
		{#each player_entries as [name, p] (name)}
			<li class="row" class:dim={!p.connected}>
				<span class="avatar" style:background={playerColor(name)}>{playerInitial(name)}</span>
				<span class="name">
					{name}
					{#if name === room.player}<em>({m.you()})</em>{/if}
				</span>
				{#if p.grants.includes('Moderate')}
					<span class="tag">
						🛡️{p.grants.includes('Play') ? '🎮' : ''}
						{m.mod_actions()}
					</span>
				{/if}
			</li>
		{/each}
	</ul>

	{#if has('Moderate')}
		<Button size="lg" disabled={!can_start} onclick={() => room.send?.({ kind: 'StartGame' })}>
			▶ {m.start_game()}
		</Button>
	{:else}
		<p class="muted center">{m.waiting_for_host()}</p>
	{/if}
</section>

<style>
	.lobby {
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
	.center {
		text-align: center;
	}
	.roster {
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
		font-size: clamp(1rem, 2.8cqi, 1.6rem);
	}
	.dim {
		opacity: 0.4;
	}
	.avatar {
		width: clamp(2.25rem, 6cqi, 3.5rem);
		height: clamp(2.25rem, 6cqi, 3.5rem);
		border-radius: var(--radius-full);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--color-text-inverse);
		font-weight: 700;
		font-family: var(--font-heading);
		font-size: clamp(0.95rem, 2.4cqi, 1.4rem);
		flex-shrink: 0;
		text-transform: uppercase;
	}
	.name {
		flex: 1;
	}
	.name em {
		color: var(--color-text-muted);
		font-style: normal;
	}
	.tag {
		font-size: clamp(0.75rem, 1.8cqi, 1rem);
		color: var(--color-accent);
		border: var(--border-width) var(--border-style) var(--color-accent);
		border-radius: var(--radius-full);
		padding: 0 0.4rem;
	}
</style>
