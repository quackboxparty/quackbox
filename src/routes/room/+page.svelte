<script lang="ts">
	import { goto } from '$app/navigation';
	import { ScrollArea } from 'bits-ui';
	import { api } from '$lib/api';
	import Button from '$lib/components/Button.svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import TextInput from '$lib/components/TextInput.svelte';
	import GameCard from '$lib/components/GameCard.svelte';
	import { m } from '$lib/paraglide/messages';
	import { toast } from '$lib/toast.svelte';

	let hostOpen = $state(false);
	let secret = $state('');
	let selectedGame = $state<string | null>(null);

	function pick(gameId: string) {
		selectedGame = selectedGame === gameId ? null : gameId;
	}

	async function create() {
		if (!selectedGame) {
			toast.error(m.no_game_selected());
			return;
		}

		const result = await api.room.create(secret, selectedGame);

		if (!result.ok) {
			toast.error(m.error_generic());
			return;
		}

		await goto(`/room/${result.value.join_code}`);
	}
</script>

<section class="room">
	<h2 class="room-title">{m.games()}</h2>

	<ScrollArea.Root class="games-scroll" type="hover">
		<ScrollArea.Viewport class="games-viewport">
			{#await api.games.list()}
				<p class="state">…</p>
			{:then result}
				{#if !result.ok}
					<p class="state">{m.error_generic()}</p>
				{:else if result.value.length === 0}
					<p class="state">No games yet.</p>
				{:else}
					<div class="grid">
						{#each result.value as game (game.id)}
							<GameCard {game} selected={selectedGame === game.id} onclick={() => pick(game.id)} />
						{/each}
					</div>
				{/if}
			{/await}
		</ScrollArea.Viewport>
		<ScrollArea.Scrollbar orientation="vertical" class="games-bar">
			<ScrollArea.Thumb class="games-thumb" />
		</ScrollArea.Scrollbar>
	</ScrollArea.Root>

	<div class="room-foot">
		<Button size="lg" disabled={!selectedGame} onclick={() => (hostOpen = true)}>{m.host()}</Button>
	</div>
</section>

<Dialog bind:open={hostOpen} title={m.host()}>
	<form
		onsubmit={(e) => {
			e.preventDefault();
			create();
		}}
	>
		<TextInput bind:value={secret} placeholder="Secret" />
	</form>
	<Button size="lg" onclick={create}>{m.create_room()}</Button>
</Dialog>

<style>
	.room {
		height: 100%;
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
		width: 100%;
		max-width: 64rem;
		margin-inline: auto;
		padding: var(--space-6);
	}
	.room-title {
		flex: none;
		font-family: var(--font-heading);
	}
	:global(.games-scroll) {
		flex: 1;
		min-height: 0;
	}
	:global(.games-viewport) {
		height: 100%;
		padding-top: 10px;
		padding-left: 10px;
		/* clear the absolutely-positioned scrollbar on the right edge */
		padding-right: var(--space-2);
	}
	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(18rem, 1fr));
		gap: var(--space-4);
		align-items: stretch;
	}
	.state {
		color: var(--color-text-muted);
		padding: var(--space-8);
		text-align: center;
	}
	.room-foot {
		flex: none;
	}
	:global(.room-foot .btn) {
		width: 100%;
	}

	/* bits-ui ScrollArea renders these via class props, so style them globally.
	   The vertical scrollbar is required — without it bits keeps the viewport
	   overflow:hidden and nothing scrolls. */
	:global(.games-bar) {
		position: absolute;
		width: 0.5rem;
		/* vertical-only: thumb width normally comes from an x-axis bar's var */
		--bits-scroll-area-thumb-width: 100%;
	}
	:global(.games-thumb) {
		background: var(--color-text-muted);
		border-radius: var(--radius-full);
	}
</style>
