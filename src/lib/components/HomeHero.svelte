<script lang="ts">
	import Logo from '$lib/components/Logo.svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import CodeInput from '$lib/components/CodeInput.svelte';
	import { m } from '$lib/paraglide/messages';
	import { goto } from '$app/navigation';
	import { api } from '$lib/api';
	import { toast } from '$lib/toast.svelte';
	import { lastSession, type RoomSession } from '$lib/session';
	import Button from './Button.svelte';
	import { onMount } from 'svelte';

	let joinOpen = $state(false);
	let joinCode = $state('');
	let recentSession: RoomSession | null = $state(null);

	async function join() {
		if (joinCode.length !== 6) return;

		const result = await api.room.exists(joinCode);

		if (!result.ok) {
			toast.error(m.error_generic());
			return;
		}
		if (!result.value) {
			toast.error(m.room_not_found());
			return;
		}

		await goto(`/room/${joinCode}`);
	}

	async function room() {
		await goto('/room');
	}

	async function rejoin() {
		if (recentSession) await goto(`/room/${recentSession.room}`);
	}

	onMount(() => {
		recentSession = lastSession();
	});
</script>

<section class="hero">
	<Logo size="lg" stacked />
	<p class="tagline">{m.tagline()}</p>

	<div class="actions">
		<Button size="xl" onclick={() => (joinOpen = true)}>{m.join_game()}</Button>
		<Button variant="secondary" size="xl" onclick={room}>{m.host()}</Button>
	</div>
</section>

<Dialog bind:open={joinOpen} title={m.join_game()} description={m.enter_join_code()}>
	<CodeInput bind:value={joinCode} onComplete={join} />
	<Button disabled={joinCode.length !== 6} onclick={join}>
		{m.join()}
	</Button>
	{#if recentSession}
		<Button variant="secondary" onclick={rejoin}>{m.rejoin()} {recentSession.room}</Button>
	{/if}
</Dialog>

<style>
	.hero {
		min-height: 100%;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-6);
		text-align: center;
		padding: var(--space-12) var(--space-6);
	}
	.tagline {
		margin: 0;
		color: var(--color-text-muted);
		font-size: calc(1rem * var(--font-scale));
	}
	.actions {
		display: flex;
		gap: var(--space-4);
		flex-wrap: wrap;
		justify-content: center;
	}
</style>
