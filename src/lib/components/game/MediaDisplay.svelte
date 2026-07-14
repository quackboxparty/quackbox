<script lang="ts">
	import type { MediaView } from '$lib/bindings/Protocol';

	let { media }: { media: MediaView } = $props();

	// Url video/audio use a media fragment (#t=start,end); YouTube uses
	// start/end query params (whole seconds only).
	const src = $derived.by(() => {
		if (media.src.kind === 'Youtube') {
			const params = new URLSearchParams();
			if (media.kind !== 'Image') {
				if (media.start_ms != null)
					params.set('start', String(Math.floor(media.start_ms / 1000)));
				if (media.end_ms != null) params.set('end', String(Math.ceil(media.end_ms / 1000)));
			}
			const query = params.size ? `?${params}` : '';
			return `https://www.youtube-nocookie.com/embed/${media.src.value}${query}`;
		}
		if (media.kind === 'Image' || (media.start_ms == null && media.end_ms == null)) {
			return media.src.value;
		}
		const start = (media.start_ms ?? 0) / 1000;
		return media.end_ms == null
			? `${media.src.value}#t=${start}`
			: `${media.src.value}#t=${start},${media.end_ms / 1000}`;
	});
</script>

{#if media.src.kind === 'Youtube'}
	<iframe
		class="embed"
		{src}
		title={media.alt ?? 'media'}
		allow="autoplay; encrypted-media"
		allowfullscreen
	></iframe>
{:else if media.kind === 'Image'}
	<img class="visual" {src} alt={media.alt ?? ''} width={media.width} height={media.height} />
{:else if media.kind === 'Video'}
	<!-- svelte-ignore a11y_media_has_caption -- quiz media has no caption tracks -->
	<video class="visual" {src} controls width={media.width} height={media.height}></video>
{:else}
	<audio class="audio" {src} controls></audio>
{/if}

<style>
	.visual {
		max-width: min(40rem, 100%);
		max-height: 40vh;
		width: auto;
		height: auto;
		border-radius: var(--radius-md);
	}
	.embed {
		width: min(40rem, 100%);
		aspect-ratio: 16 / 9;
		border: 0;
		border-radius: var(--radius-md);
	}
	.audio {
		width: min(28rem, 100%);
	}
</style>
