<script lang="ts">
	import type { Snippet } from 'svelte';
	import { resolve } from '$app/paths';
	import {
		themes,
		getTheme,
		setTheme,
		setSystemTheme,
		hasStoredTheme,
		type ThemeId
	} from '$lib/themes';
	import { locales, setLocale, type Locale } from '$lib/paraglide/runtime';
	import { m } from '$lib/paraglide/messages';
	import { currentLocale, themeLabel } from '$lib/i18n.svelte';
	import Logo from '$lib/components/Logo.svelte';

	let { children }: { children: Snippet } = $props();

	let open = $state(false);
	let theme = $state<ThemeId>(getTheme());
	let usingSystem = $state(!hasStoredTheme());

	function pickTheme(id: ThemeId) {
		theme = id;
		usingSystem = false;
		setTheme(id);
	}
	function pickSystemTheme() {
		usingSystem = true;
		setSystemTheme();
		theme = getTheme();
	}
	function pickLang(loc: Locale) {
		void setLocale(loc);
	}
</script>

<div class="shell">
	<header class="navbar">
		<a class="brand" href={resolve('/', {})}>
			<Logo showWordmark size="sm" />
		</a>
		<button class="burger" aria-label={m.menu()} aria-expanded={open} onclick={() => (open = !open)}>
			<span class="burger-bar"></span>
			<span class="burger-bar"></span>
			<span class="burger-bar"></span>
		</button>
	</header>

	<main class="main">
		{@render children()}
	</main>
</div>

<svelte:window
	on:keydown={(e) => {
		if (open && e.key === 'Escape') open = false;
	}}
/>

{#if open}
	<button class="scrim" aria-label="Close menu" onclick={() => (open = false)}></button>
	<aside class="drawer" aria-label="Settings">
		<div class="drawer-head">
			<h2 class="drawer-title">{m.settings()}</h2>
			<button class="drawer-close" aria-label={m.close()} onclick={() => (open = false)}>✕</button>
		</div>

		<div class="drawer-section">
			<h3 class="section-label">🎨 {m.theme()}</h3>
			<div class="chip-row">
				<button
					class="chip"
					class:chip-active={usingSystem}
					onclick={() => {
						pickSystemTheme();
					}}
				>
					<span class="chip-emoji">🖥️</span>
					{m.theme_system()}
				</button>
				{#each Object.values(themes) as t (t.id)}
					<button
						class="chip"
						class:chip-active={!usingSystem && theme === t.id}
						onclick={() => {
							pickTheme(t.id);
						}}
					>
						<span class="chip-emoji">{t.emojis[0]}</span>
						{themeLabel(t.id)}
					</button>
				{/each}
			</div>
		</div>

		<div class="drawer-section">
			<h3 class="section-label">🌐 {m.language()}</h3>
			<div class="chip-row">
				{#each locales as loc (loc)}
					<button
						class="chip"
						class:chip-active={currentLocale() === loc}
						onclick={() => {
							pickLang(loc);
						}}
					>
						{loc.toUpperCase()}
					</button>
				{/each}
			</div>
		</div>
	</aside>
{/if}

<style>
	.shell {
		min-height: 100vh;
		min-height: 100dvh;
		display: flex;
		flex-direction: column;
	}
	.navbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-3) var(--space-6);
		border-bottom: var(--border-width) var(--border-style) var(--border-color);
		background: var(--bg-surface);
		position: sticky;
		top: 0;
		z-index: 10;
	}
	.brand {
		display: flex;
		align-items: center;
		text-decoration: none;
	}
	.burger {
		display: flex;
		flex-direction: column;
		justify-content: center;
		gap: 5px;
		width: 2.5rem;
		height: 2.5rem;
		padding: 0 var(--space-2);
		border: var(--border-width) var(--border-style) var(--border-color);
		border-radius: var(--radius-md);
		background: var(--bg-surface);
		cursor: pointer;
	}
	.burger-bar {
		display: block;
		height: 2px;
		width: 100%;
		background: var(--color-text);
	}
	.main {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: var(--space-12) var(--space-6);
	}

	.scrim {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.5);
		border: none;
		z-index: 40;
		cursor: default;
	}
	.drawer {
		position: fixed;
		top: 0;
		right: 0;
		bottom: 0;
		width: min(20rem, 85vw);
		background: var(--bg-surface-elevated);
		border-left: var(--border-width) var(--border-style) var(--border-color);
		box-shadow: var(--shadow-lg);
		padding: var(--space-6);
		z-index: 50;
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
		animation: slide-in 0.2s var(--easing);
	}
	@keyframes slide-in {
		from {
			transform: translateX(100%);
		}
		to {
			transform: translateX(0);
		}
	}
	.drawer-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}
	.drawer-title {
		font-family: var(--font-heading);
		font-size: calc(1.25rem * var(--font-scale));
		margin: 0;
	}
	.drawer-close {
		background: none;
		border: none;
		font-size: 1.25rem;
		color: var(--color-text-muted);
		cursor: pointer;
	}
	.drawer-section {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}
	.section-label {
		font-family: var(--font-heading);
		font-size: calc(0.75rem * var(--font-scale));
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--color-text-muted);
		margin: 0;
	}
	.chip-row {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
	}
	.chip {
		display: inline-flex;
		align-items: center;
		gap: var(--space-1);
		padding: var(--space-2) var(--space-3);
		border: var(--border-width) var(--border-style) var(--border-color);
		border-radius: var(--radius-full);
		background: var(--bg-surface);
		color: var(--color-text);
		font-family: var(--font-body);
		font-size: calc(0.875rem * var(--font-scale));
		cursor: pointer;
	}
	.chip-emoji {
		font-size: 1.1em;
	}
	.chip-active {
		border-color: var(--color-primary);
		color: var(--color-primary);
		font-weight: 600;
	}

	@media (max-width: 480px) {
		.navbar {
			padding: var(--space-3);
		}
	}
</style>
