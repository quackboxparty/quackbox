<script lang="ts">
	import Logo from '$lib/components/Logo.svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import CodeInput from '$lib/components/CodeInput.svelte';
	import { m } from '$lib/paraglide/messages';

	let joinOpen = $state(false);
	let hostOpen = $state(false);
	let joinCode = $state('');

	function join() {
		if (joinCode.length !== 6) return;
		console.log('join', joinCode);
	}
</script>

<section class="hero">
	<Logo size="lg" stacked />
	<p class="tagline">{m.tagline()}</p>

	<div class="actions">
		<button class="btn btn-primary" onclick={() => (joinOpen = true)}>{m.join_game()}</button>
		<button class="btn btn-secondary" onclick={() => (hostOpen = true)}>{m.host()}</button>
	</div>
</section>

<Dialog bind:open={joinOpen} title={m.join_game()} description={m.enter_join_code()}>
	<CodeInput bind:value={joinCode} onComplete={join} />
	<button class="btn btn-primary" disabled={joinCode.length !== 6} onclick={join}>{m.join()}</button>
</Dialog>
<Dialog bind:open={hostOpen} title={m.host()} />

<style>
	.hero {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-6);
		text-align: center;
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
	.btn {
		font-family: var(--font-heading);
		font-size: calc(1.125rem * var(--font-scale));
		padding: var(--space-4) var(--space-8);
		border-radius: var(--radius-lg);
		border: var(--border-width) var(--border-style) transparent;
		cursor: pointer;
		transition: transform var(--duration-fast) var(--easing);
	}
	.btn:active {
		transform: scale(0.97);
	}
	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
	.btn-primary {
		background: var(--color-primary);
		color: var(--color-text-inverse);
	}
	.btn-secondary {
		background: var(--bg-surface-elevated);
		color: var(--color-text);
		border-color: var(--border-color);
	}
</style>
