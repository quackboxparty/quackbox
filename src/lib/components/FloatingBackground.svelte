<script lang="ts">
	import { themes, type ThemeId } from '$lib/themes';
	import { browser } from '$app/environment';

	interface Particle {
		emoji: string;
		x: number;
		y: number;
		size: number;
		duration: number;
		delay: number;
		drift: number;
		opacity: number;
	}

	const COUNT = 16;

	/** Deterministic PRNG so particles don't jump on re-render */
	function seededRandom(seed: number): () => number {
		let s = seed;
		return () => {
			s = (s * 16807) % 2147483647;
			return (s - 1) / 2147483646;
		};
	}

	function buildParticles(themeId: ThemeId): Particle[] {
		const emojis = themes[themeId].emojis;
		const rng = seededRandom(42);
		return Array.from({ length: COUNT }, (_, i) => ({
			emoji: emojis[i % emojis.length] ?? '🦆',
			x: rng() * 100,
			y: rng() * 100,
			size: 1.5 + rng() * 2,
			duration: 15 + rng() * 20,
			delay: -(rng() * 25),
			drift: -10 + rng() * 20,
			opacity: 0.12 + rng() * 0.18
		}));
	}

	let currentTheme = $state<ThemeId>('modern');
	let particles = $state<Particle[]>(buildParticles('modern'));

	$effect(() => {
		if (!browser) return;

		const readTheme = (): ThemeId => {
			const attr = document.documentElement.getAttribute('data-theme');
			if (attr && attr in themes) return attr as ThemeId;
			return 'modern';
		};

		currentTheme = readTheme();

		const observer = new MutationObserver(() => {
			const next = readTheme();
			if (next !== currentTheme) currentTheme = next;
		});
		observer.observe(document.documentElement, {
			attributes: true,
			attributeFilter: ['data-theme']
		});
		return () => observer.disconnect();
	});

	// Rebuild particles when theme changes
	$effect(() => {
		particles = buildParticles(currentTheme);
	});
</script>

<div class="float-bg" aria-hidden="true">
	{#each particles as p, i (i)}
		<span
			class="particle"
			style="
				left: {p.x}%;
				top: {p.y}%;
				font-size: {p.size}rem;
				animation-duration: {p.duration}s;
				animation-delay: {p.delay}s;
				--drift: {p.drift}deg;
				opacity: {p.opacity};
			"
		>
			{p.emoji}
		</span>
	{/each}
</div>

<style>
	.float-bg {
		position: fixed;
		inset: 0;
		pointer-events: none;
		overflow: hidden;
		z-index: -1;
	}

	.particle {
		position: absolute;
		display: block;
		animation: float linear infinite;
		will-change: transform;
	}

	@keyframes float {
		0% {
			transform: translateY(0) translateX(0) rotate(0deg);
		}
		25% {
			transform: translateY(-25vh) translateX(5vw) rotate(calc(var(--drift) * 0.5));
		}
		50% {
			transform: translateY(-50vh) translateX(-3vw) rotate(var(--drift));
		}
		75% {
			transform: translateY(-75vh) translateX(4vw) rotate(calc(var(--drift) * 1.5));
		}
		100% {
			transform: translateY(-100vh) translateX(0) rotate(calc(var(--drift) * 2));
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.particle {
			animation: none;
		}
	}
</style>
