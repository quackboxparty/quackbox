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
	import Drawer from '$lib/components/Drawer.svelte';
	import Button from '$lib/components/Button.svelte';
	import { playerColor, playerInitial } from '$lib/playerUi';
	import { room, has } from '$lib/room.svelte';
	import type { Grant } from '$lib/bindings/Grants';
	import { DropdownMenu } from 'bits-ui';
	import { toast } from '$lib/toast.svelte';

	let { children }: { children: Snippet } = $props();

	let open = $state(false);
	let playersOpen = $state(false);
	let modOpen = $state(false);
	let tab = $state<'players' | 'scoreboard'>('players');

	const phase = $derived.by(() => {
		const stage = room.gamestate?.stage;
		return stage?.kind === 'GridQuiz' ? stage.phase : undefined;
	});
	const sortedPlayers = $derived(
		Object.entries(room.gamestate?.players ?? {}).sort(([, a], [, b]) => b.score - a.score)
	);

	const allGrants: Grant[] = ['Play', 'Present', 'Moderate'];
	const grantLabels: Record<Grant, () => string> = {
		Play: m.grant_play,
		Present: m.grant_present,
		Moderate: m.grant_moderate
	};

	function kick(player: string) {
		room.send?.({ kind: 'Kick', player });
	}
	function toggleGrant(player: string, current: Grant[], g: Grant) {
		const grants = current.includes(g) ? current.filter((x) => x !== g) : [...current, g];
		room.send?.({ kind: 'Grant', player, grants });
	}
	function endGame() {
		room.send?.({ kind: 'EndGame' });
		toast.success(m.end_game_sent());
		modOpen = false;
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
					aria-label={m.players_and_scoreboard()}
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
			{#if room.code && has('Moderate')}
				<button
					class="icon-btn"
					aria-label={m.mod_actions()}
					aria-expanded={modOpen}
					onclick={() => (modOpen = !modOpen)}
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
						<path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
					</svg>
				</button>
			{/if}
			<button
				class="icon-btn"
				aria-label={m.settings()}
				aria-expanded={open}
				onclick={() => (open = !open)}
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
					<circle cx="9" cy="9" r="3" />
					<path
						d="M9 2.8v1.7M9 13.5v1.7M2.8 9h1.7M13.5 9h1.7M4.6 4.6l1.2 1.2M12.2 12.2l1.2 1.2M13.4 4.6l-1.2 1.2M5.8 12.2l-1.2 1.2"
					/>
					<circle cx="18" cy="17" r="2" />
					<path
						d="M18 12.6v1.4M18 20v1.4M13.6 17h1.4M21 17h1.4M14.9 13.9l1 1M20.1 19.1l1 1M21.1 13.9l-1 1M15.9 19.1l-1 1"
					/>
				</svg>
			</button>
		</div>
	</header>

	<main class="main">
		{@render children()}
	</main>
</div>

<Drawer bind:open title={m.settings()}>
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
</Drawer>

<Drawer bind:open={playersOpen} title={m.players_and_scoreboard()}>
	{#snippet header()}
		<div class="tabs" role="tablist">
			<button
				class="tab"
				role="tab"
				aria-selected={tab === 'players'}
				class:tab-active={tab === 'players'}
				onclick={() => (tab = 'players')}>{m.players()}</button
			>
			<button
				class="tab"
				role="tab"
				aria-selected={tab === 'scoreboard'}
				class:tab-active={tab === 'scoreboard'}
				onclick={() => (tab = 'scoreboard')}>{m.scoreboard()}</button
			>
		</div>
	{/snippet}

	{#if tab === 'players'}
		<div class="drawer-section">
			<ul class="player-list">
				{#each Object.entries(room.gamestate?.players ?? {}) as [player, view] (player)}
					<li class="player-row">
						<span class="player-avatar" style:background={playerColor(player)}
							>{playerInitial(player)}</span
						>
						<span class="player-name">
							{player}
							{#if room.player === player}
								<span class="player-you">({m.you()})</span>
							{/if}
						</span>
						{#if view.grants.includes('Moderate')}
							<span class="mod-badge" title="Moderator">🛡️ Mod</span>
						{/if}
						{#if has('Moderate') && player !== room.player}
							<DropdownMenu.Root>
								<DropdownMenu.Trigger
									class="player-menu-btn"
									aria-label={m.player_actions({ name: player })}
									title={m.player_actions({ name: player })}
								>
									<svg class="dots-icon" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
										<circle cx="5" cy="12" r="2" />
										<circle cx="12" cy="12" r="2" />
										<circle cx="19" cy="12" r="2" />
									</svg>
								</DropdownMenu.Trigger>
								<DropdownMenu.Portal>
									<DropdownMenu.Content class="menu-list" align="end" sideOffset={4}>
										{#each allGrants as g (g)}
											<DropdownMenu.CheckboxItem
												class="menu-item"
												checked={view.grants.includes(g)}
												closeOnSelect={false}
												onCheckedChange={() => {
													toggleGrant(player, view.grants, g);
												}}
											>
												<span class="menu-check">{view.grants.includes(g) ? '✓' : ''}</span>
												{grantLabels[g]()}
											</DropdownMenu.CheckboxItem>
										{/each}
										<DropdownMenu.Item
											class="menu-item menu-danger"
											onSelect={() => {
												kick(player);
											}}
										>
											<span class="menu-check">✕</span>
											{m.kick_player({ name: player })}
										</DropdownMenu.Item>
									</DropdownMenu.Content>
								</DropdownMenu.Portal>
							</DropdownMenu.Root>
						{/if}
					</li>
				{/each}
			</ul>
		</div>
	{:else}
		<div class="drawer-section">
			{#if phase === 'lobby'}
				<p class="empty-state">{m.game_not_started()}</p>
			{:else}
				<ul class="player-list">
					{#each sortedPlayers as [player, view], i (player)}
						<li class="player-row">
							<span class="rank">{i + 1}</span>
							<span class="player-avatar" style:background={playerColor(player)}
								>{playerInitial(player)}</span
							>
							<span class="player-name">
								{player}
								{#if room.player === player}
									<span class="player-you">({m.you()})</span>
								{/if}
							</span>
							<span class="score">{view.score}</span>
						</li>
					{/each}
				</ul>
			{/if}
		</div>
	{/if}
</Drawer>

<Drawer bind:open={modOpen} title={m.mod_actions()}>
	<div class="drawer-section">
		<Button variant="danger" onclick={endGame}>{m.end_game()}</Button>
	</div>
	<!-- overrule/revisie judgments + grant management slots land when #15 is wired -->
</Drawer>

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
	.main {
		flex: 1;
		min-height: 0;
		overflow-y: auto;
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
	/* bits-ui renders these (Content portaled to body) — scoped selectors can't reach them */
	:global(.player-menu-btn) {
		margin-left: auto;
		width: 1.5rem;
		height: 1.5rem;
		border: none;
		border-radius: var(--radius-sm);
		background: transparent;
		color: var(--color-text-muted);
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}
	:global(.player-menu-btn:hover),
	:global(.player-menu-btn[data-state='open']) {
		background: var(--bg-muted, var(--border-color));
		color: var(--color-text);
	}
	:global(.player-menu-btn .dots-icon) {
		width: 1.1rem;
		height: 1.1rem;
	}
	:global(.menu-list) {
		z-index: 60;
		min-width: 10rem;
		padding: var(--space-1);
		border: var(--border-width) var(--border-style) var(--border-color);
		border-radius: var(--radius-md);
		background: var(--bg-surface);
		box-shadow: 0 4px 12px rgb(0 0 0 / 0.15);
		display: flex;
		flex-direction: column;
		outline: none;
	}
	:global(.menu-item) {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-2) var(--space-3);
		border-radius: var(--radius-sm);
		color: var(--color-text);
		font-family: var(--font-body);
		font-size: calc(0.875rem * var(--font-scale));
		cursor: pointer;
		white-space: nowrap;
		outline: none;
	}
	:global(.menu-item[data-highlighted]) {
		background: var(--bg-muted, var(--border-color));
	}
	:global(.menu-check) {
		width: 1em;
		flex-shrink: 0;
	}
	:global(.menu-danger) {
		color: var(--color-danger);
	}
	:global(.menu-danger[data-highlighted]) {
		background: var(--color-danger);
		color: var(--color-text-inverse);
	}
	.tabs {
		display: flex;
		gap: var(--space-1);
		flex: 1;
	}
	.tab {
		flex: 1;
		padding: var(--space-2) var(--space-3);
		border: none;
		border-bottom: 2px solid transparent;
		background: transparent;
		color: var(--color-text-muted);
		font-family: var(--font-body);
		font-size: calc(0.875rem * var(--font-scale));
		font-weight: 600;
		cursor: pointer;
	}
	.tab-active {
		color: var(--color-primary);
		border-bottom-color: var(--color-primary);
	}
	.rank {
		font-family: var(--font-heading);
		font-weight: 700;
		color: var(--color-text-muted);
		min-width: 1.5rem;
		font-size: calc(0.9rem * var(--font-scale));
	}
	.score {
		margin-left: auto;
		font-family: var(--font-heading);
		font-weight: 700;
		font-size: calc(1rem * var(--font-scale));
	}
	.empty-state {
		margin: 0;
		padding: var(--space-6) var(--space-3);
		text-align: center;
		color: var(--color-text-muted);
		font-family: var(--font-body);
		font-size: calc(0.9rem * var(--font-scale));
	}

	@media (max-width: 480px) {
		.navbar {
			padding: var(--space-3);
		}
	}
</style>
