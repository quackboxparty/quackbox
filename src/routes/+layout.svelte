<script lang="ts">
	import type { Snippet } from 'svelte';

import { getTextDirection, getLocale } from '$lib/paraglide/runtime';
	import { initReactiveLocale } from '$lib/i18n.svelte';

	import favicon from '$lib/assets/favicon.svg';
	import { initTheme } from '$lib/themes';
	import FloatingBackground from '$lib/components/FloatingBackground.svelte';
	import Toaster from '$lib/components/Toaster.svelte';
	import AppShell from '$lib/components/AppShell.svelte';

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

	initReactiveLocale();
	initTheme();

	$effect(() => {
		document.documentElement.lang = getLocale();
		document.documentElement.dir = getTextDirection();
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} type="image/svg+xml" />
	<link rel="icon" href="/favicon-32.png" sizes="32x32" />
	<link rel="apple-touch-icon" href="/favicon-180.png" />
</svelte:head>

<FloatingBackground />
<AppShell>
	{@render children()}
</AppShell>
<Toaster />
