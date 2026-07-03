<script lang="ts">
	import duck from '$lib/assets/duck.svg';
	import duckRetro from '$lib/assets/duck-retro.png';
	import duckKawaii from '$lib/assets/duck-kawaii.svg';
	import wordmarkSvg from '$lib/assets/wordmark.svg?raw';
	import { onDestroy } from 'svelte';
	import { m } from '$lib/paraglide/messages';

	let {
		showWordmark = true,
		size = 'md',
		stacked = false
	}: { showWordmark?: boolean; size?: 'sm' | 'md' | 'lg'; stacked?: boolean } = $props();

	let quackEl: HTMLAudioElement | null = null;
	let quacking = $state(false);
	let quackTimer: ReturnType<typeof setTimeout> | null = null;

	function quack() {
		quackEl ??= new Audio('/sounds/quack.mp3');
		quackEl.currentTime = 0;
		void quackEl.play().catch(() => undefined);
		quacking = true;
		if (quackTimer) clearTimeout(quackTimer);
		quackTimer = setTimeout(() => (quacking = false), 900);
	}

	onDestroy(() => {
		if (quackTimer) clearTimeout(quackTimer);
	});
</script>

<div
	class="logo"
	class:logo-sm={size === 'sm'}
	class:logo-lg={size === 'lg'}
	class:logo-stacked={stacked}
>
	<button
		type="button"
		class="logo-duck-frame"
		class:is-quacking={quacking}
		onclick={quack}
		aria-label={m.quack()}
	>
		<img class="logo-duck" src={duck} alt="" />
		<img class="logo-duck logo-duck-retro" src={duckRetro} alt="" />
		<img class="logo-duck logo-duck-kawaii" src={duckKawaii} alt="" />
		<span class="logo-quack" aria-hidden="true">
			<i></i>
			<i></i>
			<i></i>
		</span>
	</button>
	{#if showWordmark}
		<!-- eslint-disable-next-line svelte/no-at-html-tags -- trusted local SVG asset -->
		<div class="logo-wordmark" role="img" aria-label="Quackbox">{@html wordmarkSvg}</div>
	{/if}
</div>

<style>
	.logo {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}
	.logo-stacked {
		flex-direction: column;
		text-align: center;
	}
	.logo-sm .logo-duck {
		width: 2rem;
	}
	.logo-sm .logo-duck-frame {
		font-size: 2rem;
	}
	.logo-sm .logo-wordmark {
		width: 8rem;
	}
	.logo-lg .logo-duck {
		width: 6rem;
	}
	.logo-lg .logo-duck-frame {
		font-size: 6rem;
	}
	.logo-lg .logo-wordmark {
		width: 18rem;
		max-width: 80vw;
	}
	.logo-duck-frame {
		position: relative;
		display: inline-flex;
		font-size: 4rem;
		padding: 0;
		border: none;
		background: none;
		cursor: pointer;
	}
	.logo-duck {
		width: 4rem;
		height: auto;
		flex-shrink: 0;
		transition: filter var(--duration-normal) var(--easing);
	}

	/* Pixel duck only appears on the retro theme; kawaii duck only on kawaii. */
	.logo-duck-retro {
		display: none;
		image-rendering: pixelated;
	}
	.logo-duck-kawaii {
		display: none;
	}
	:global([data-theme='retro']) .logo-duck:not(.logo-duck-retro),
	:global([data-theme='kawaii']) .logo-duck:not(.logo-duck-kawaii) {
		display: none;
	}
	:global([data-theme='retro']) .logo-duck-retro,
	:global([data-theme='kawaii']) .logo-duck-kawaii {
		display: block;
	}
	.logo-wordmark {
		width: 14rem;
		color: var(--color-text);
	}
	.logo-wordmark :global(svg) {
		width: 100%;
		height: auto;
		display: block;
	}

	/* Per-theme duck tinting via whole-image filters. Wordmark left plain. */
	:global([data-theme='modern-dark']) .logo-duck {
		filter: brightness(1.05);
	}
	:global([data-theme='neon']) .logo-duck {
		filter: saturate(1.6) drop-shadow(0 0 12px var(--color-primary));
	}
	:global([data-theme='retro']) .logo-duck-retro {
		filter: saturate(1.4) contrast(1.2);
	}
	:global([data-theme='medieval']) .logo-duck {
		filter: sepia(0.4) saturate(0.85) brightness(0.95);
	}
	:global([data-theme='chalkboard']) .logo-duck {
		filter: grayscale(1) brightness(1.5) contrast(0.8);
	}
	:global([data-theme='kawaii']) .logo-duck {
		filter: saturate(0.7) brightness(1.15) contrast(0.9);
	}
	:global([data-theme='western']) .logo-duck {
		filter: sepia(0.7) saturate(1.3) brightness(0.95);
	}
	:global([data-theme='wizard']) .logo-duck {
		animation: wizard-pulse 7s ease-in-out infinite;
	}
	@keyframes wizard-pulse {
		0%,
		100% {
			filter: saturate(1.2) brightness(1) drop-shadow(0 0 3px var(--color-primary))
				drop-shadow(0 0 4px var(--color-accent));
		}
		50% {
			filter: saturate(1.4) brightness(1.12) drop-shadow(0 0 10px var(--color-primary))
				drop-shadow(0 0 18px var(--color-accent));
		}
	}

	.logo-quack {
		position: absolute;
		left: 100%;
		top: calc(42% - 0.25em);
		pointer-events: none;
		opacity: 0;
		color: var(--color-primary);
		transition: opacity var(--duration-normal) var(--easing);
	}
	.logo-duck-frame.is-quacking .logo-quack {
		opacity: 1;
	}
	.logo-quack i {
		position: absolute;
		top: 50%;
		left: 0;
		display: block;
		width: 0.22em;
		height: 0.045em;
		border-radius: 0.03em;
		background: currentColor;
		transform-origin: left center;
	}
	.logo-quack i:nth-child(1) {
		transform: rotate(0deg);
	}
	.logo-quack i:nth-child(2) {
		transform: rotate(15deg);
		margin-top: 0.07em;
	}
	.logo-quack i:nth-child(3) {
		transform: rotate(-15deg);
		margin-top: -0.07em;
	}

	@media (prefers-reduced-motion: reduce) {
		:global([data-theme='wizard']) .logo-duck {
			animation: none;
			filter: saturate(1.2) drop-shadow(0 0 8px var(--color-accent));
		}
		.logo-quack {
			transition: none;
		}
	}
</style>
