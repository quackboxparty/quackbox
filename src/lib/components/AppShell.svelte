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
	import { room } from '$lib/room.svelte';

	let { children }: { children: Snippet } = $props();

	let open = $state(false);
	let playersOpen = $state(false);

	// Stable per-player avatar color: hash name -> one of the themed token colors.
	const AVATAR_COLORS = ['primary', 'secondary', 'accent', 'success', 'warning', 'danger'] as const;
	function playerColor(name: string): string {
		let h = 0;
		for (let i = 0; i < name.length; i++) h = (h * 31 + name.charCodeAt(i)) >>> 0;
		return `var(--color-${AVATAR_COLORS[h % AVATAR_COLORS.length]})`;
	}
	function initial(name: string): string {
		return name.trim().charAt(0).toUpperCase() || '?';
	}
	function kick(player: string) {
		room.send?.({ kind: 'Kick', player });
	}
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
		<div class="nav-actions">
			{#if room.code}
				<button
					class="icon-btn players-toggle"
					aria-label={m.players()}
					aria-expanded={playersOpen}
					onclick={() => (playersOpen = !playersOpen)}
				>
					<svg
						class="icon"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
						aria-hidden="true"
					>
						<path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" />
						<circle cx="9" cy="7" r="4" />
						<path d="M22 21v-2a4 4 0 0 0-3-3.87" />
						<path d="M16 3.13a4 4 0 0 1 0 7.75" />
					</svg>
					{#if Object.keys(room.gamestate?.players ?? {}).length}
						<span class="badge">{Object.keys(room.gamestate?.players ?? {}).length}</span>
					{/if}
				</button>
			{/if}
			<button
				class="icon-btn burger"
				aria-label={m.menu()}
				aria-expanded={open}
				onclick={() => (open = !open)}
			>
				<span class="burger-bar"></span>
				<span class="burger-bar"></span>
				<span class="burger-bar"></span>
			</button>
		</div>
	</header>

	<main class="main">
		{@render children()}
	</main>
</div>

<svelte:window
	on:keydown={(e) => {
		if (e.key === 'Escape') {
			if (open) open = false;
			if (playersOpen) playersOpen = false;
		}
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

{#if playersOpen}
	<button class="scrim" aria-label="Close menu" onclick={() => (playersOpen = false)}></button>
	<aside class="drawer" aria-label={m.players()}>
		<div class="drawer-head">
			<h2 class="drawer-title">{m.players()}</h2>
			<button class="drawer-close" aria-label={m.close()} onclick={() => (playersOpen = false)}>
				✕
			</button>
		</div>
		<div class="drawer-section">
			<ul class="player-list">
				{#each Object.entries(room.gamestate?.players ?? {}) as [player, grants] (player)}
					<li class="player-row">
						<span class="player-avatar" style:background={playerColor(player)}
							>{initial(player)}</span
						>
						<span class="player-name">{player}{#if room.player === player}
								<span class="player-you">({m.you()})</span>
							{/if}</span>
						{#if grants.includes('Moderate')}
							<span class="mod-badge" title="Moderator">🛡️ Mod</span>
						{/if}
						{#if room.player && player !== room.player && room.gamestate?.players[room.player]?.includes('Moderate')}
							<button
								class="kick-btn"
								aria-label={m.kick_player({ name: player })}
								title={m.kick_player({ name: player })}
								onclick={() => kick(player)}>✕</button
							>
						{/if}
					</li>
				{/each}
			</ul>
		</div>
	</aside>
{/if}

<style>
	.shell {
		height: 100dvh;
		overflow: hidden;
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
	.nav-actions {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}
	.icon-btn {
		position: relative;
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		width: 2.5rem;
		height: 2.5rem;
		padding: 0 var(--space-2);
		border: var(--border-width) var(--border-style) var(--border-color);
		border-radius: var(--radius-md);
		background: var(--bg-surface);
		color: var(--color-text);
		cursor: pointer;
	}
	.burger {
		gap: 5px;
	}
	.icon {
		width: 1.25rem;
		height: 1.25rem;
	}
	.badge {
		position: absolute;
		top: -0.3rem;
		right: -0.3rem;
		min-width: 1.15rem;
		height: 1.15rem;
		padding: 0 0.3rem;
		border-radius: var(--radius-full);
		background: var(--color-primary);
		color: var(--color-text-inverse);
		font-size: calc(0.7rem * var(--font-scale));
		font-weight: 700;
		display: flex;
		align-items: center;
		justify-content: center;
		border: 2px solid var(--bg-surface);
	}
	.burger-bar {
		display: block;
		height: 2px;
		width: 100%;
		background: var(--color-text);
	}
	.main {
		flex: 1;
		min-height: 0;
		overflow-y: auto;
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
	.player-list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}
	.player-row {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-2) var(--space-3);
		border: var(--border-width) var(--border-style) var(--border-color);
		border-radius: var(--radius-md);
		background: var(--bg-surface);
	}
	.player-avatar {
		width: 2rem;
		height: 2rem;
		border-radius: var(--radius-full);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--color-text-inverse);
		font-weight: 700;
		font-family: var(--font-heading);
		font-size: calc(0.85rem * var(--font-scale));
		flex-shrink: 0;
		text-transform: uppercase;
	}
	.player-name {
		font-family: var(--font-body);
		font-size: calc(0.95rem * var(--font-scale));
	}
	.player-you {
		margin-left: 0.3em;
		color: var(--color-text-muted);
		opacity: 0.8;
		font-weight: 400;
	}
	.mod-badge {
		margin-left: auto;
		padding: 0 0.4rem;
		border-radius: var(--radius-full);
		border: var(--border-width) var(--border-style) var(--color-accent);
		color: var(--color-accent);
		background: var(--bg-surface);
		font-family: var(--font-body);
		font-size: calc(0.7rem * var(--font-scale));
		font-weight: 600;
		white-space: nowrap;
	}
	.kick-btn {
		margin-left: auto;
		width: 1.5rem;
		height: 1.5rem;
		border: none;
		border-radius: var(--radius-sm);
		background: transparent;
		color: var(--color-text-muted);
		cursor: pointer;
		font-size: calc(0.9rem * var(--font-scale));
		line-height: 1;
		display: flex;
		align-items: center;
		justify-content: center;
	}
	.kick-btn:hover {
		background: var(--color-danger);
		color: var(--color-text-inverse);
	}

	@media (max-width: 480px) {
		.navbar {
			padding: var(--space-3);
		}
	}
</style>
