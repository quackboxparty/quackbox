<script lang="ts">
	import { DropdownMenu } from 'bits-ui';
	import {
		themes,
		getTheme,
		setTheme,
		setSystemTheme,
		hasStoredTheme,
		type ThemeId
	} from '$lib/themes';

	let current = $state<ThemeId>(getTheme());
	let usingSystem = $state(hasStoredTheme());

	function switchTo(id: ThemeId) {
		current = id;
		usingSystem = false;
		setTheme(id);
	}

	function switchToSystem() {
		usingSystem = true;
		setSystemTheme();
		current = getTheme();
	}

	const resolvedLabel = $derived(usingSystem ? 'System' : themes[current].label);
</script>

<DropdownMenu.Root>
	<DropdownMenu.Trigger class="trigger">
		<span class="label">{resolvedLabel}</span>
		<span class="chevron">▾</span>
	</DropdownMenu.Trigger>

	<DropdownMenu.Portal>
		<DropdownMenu.Content class="content" sideOffset={4} align="end">
			<DropdownMenu.RadioGroup
				value={usingSystem ? '__system__' : current}
				onValueChange={(v) => {
					if (v === '__system__') switchToSystem();
					else if (v in themes) switchTo(v as ThemeId);
				}}
			>
				<DropdownMenu.RadioItem class="item" value="__system__">System</DropdownMenu.RadioItem>
				<DropdownMenu.Separator class="separator" />
				{#each Object.values(themes) as theme (theme.id)}
					<DropdownMenu.RadioItem class="item" value={theme.id}>
						{theme.label}
					</DropdownMenu.RadioItem>
				{/each}
			</DropdownMenu.RadioGroup>
		</DropdownMenu.Content>
	</DropdownMenu.Portal>
</DropdownMenu.Root>

<style>
	/*
	 * Bits UI renders internal DOM elements that Svelte's scoping can't see.
	 * Use :global() on classes passed to Bits components so styles actually apply.
	 */

	:global(.trigger) {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-2) var(--space-4);
		border: var(--border-width) var(--border-style) var(--border-color);
		border-radius: var(--radius-md);
		background: var(--bg-surface);
		color: var(--color-text);
		font-family: var(--font-body);
		font-size: calc(0.875rem * var(--font-scale));
		cursor: pointer;
		transition:
			background var(--duration-fast) var(--easing),
			border-color var(--duration-fast) var(--easing);
	}

	:global(.trigger:hover) {
		border-color: var(--color-primary);
	}

	:global(.trigger:focus-visible) {
		box-shadow: var(--focus-ring);
		outline: none;
	}

	:global(.content) {
		min-width: 10rem;
		padding: var(--space-1);
		border: var(--border-width) var(--border-style) var(--border-color);
		border-radius: var(--radius-md);
		background: var(--bg-surface);
		box-shadow: var(--shadow-lg);
		z-index: 50;
		animation: fade-in 0.15s var(--easing);
	}

	:global(.item) {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-2) var(--space-4);
		border-radius: var(--radius-sm);
		font-family: var(--font-body);
		font-size: calc(0.875rem * var(--font-scale));
		color: var(--color-text);
		cursor: pointer;
		outline: none;
		transition:
			background var(--duration-fast) var(--easing),
			color var(--duration-fast) var(--easing);
	}

	:global(.item:hover),
	:global(.item[data-highlighted]) {
		background: var(--bg-surface-elevated);
	}

	:global(.item[data-state='checked']) {
		color: var(--color-primary);
		font-weight: 600;
	}

	:global(.separator) {
		height: 1px;
		margin: var(--space-1) 0;
		background: var(--border-color);
	}

	.chevron {
		opacity: 0.5;
		font-size: calc(0.75rem * var(--font-scale));
	}

	.label {
		white-space: nowrap;
	}

	@keyframes fade-in {
		from {
			opacity: 0;
			transform: translateY(-4px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}
</style>
