<script lang="ts">
	import type { Snippet } from 'svelte';

	import favicon from '$lib/assets/favicon.svg';
	import { initTheme } from '$lib/themes';
	import FloatingBackground from '$lib/components/FloatingBackground.svelte';

	import '$lib/themes/reset.css';
	import '$lib/themes/fonts.css';
	import '$lib/themes/tokens.css';
	import '$lib/themes/modern-dark.css';
	import '$lib/themes/retro.css';
	import '$lib/themes/medieval.css';
	import '$lib/themes/neon.css';
	import '$lib/themes/chalkboard.css';
	import '$lib/themes/kawaii.css';
	import '$lib/themes/western.css';
	import '$lib/themes/wizard.css';

	let { children }: { children: Snippet } = $props();

	initTheme();
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

<FloatingBackground />
{@render children()}
