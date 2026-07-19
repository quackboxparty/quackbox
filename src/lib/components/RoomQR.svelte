<script lang="ts">
	import { Popover } from 'bits-ui';
	import QRCode from 'qrcode';
	import { m } from '$lib/paraglide/messages';
	import { room } from '$lib/room.svelte';
	import { toast } from '$lib/toast.svelte';

	let svg = $state('');
	let url = $state('');

	const code = $derived(room.code);

	$effect(() => {
		if (!code) {
			svg = '';
			url = '';
			return;
		}
		url = `${window.location.origin}/room/${code}`;
		QRCode.toString(url, {
			type: 'svg',
			margin: 1,
			width: 240,
			color: { dark: '#000000', light: '#ffffff' }
		}).then((s) => (svg = s));
	});

	async function copy() {
		try {
			await navigator.clipboard.writeText(url);
			toast.success(m.copy_link_success());
		} catch {
			toast.error(m.copy_link_failed());
		}
	}
</script>

{#if code}
	<Popover.Root>
		<Popover.Trigger class="icon-btn" aria-label={m.share_room()}>
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
				<rect x="3" y="3" width="7" height="7" rx="1" />
				<rect x="14" y="3" width="7" height="7" rx="1" />
				<rect x="3" y="14" width="7" height="7" rx="1" />
				<line x1="14" y1="14" x2="14" y2="17" />
				<line x1="14" y1="20" x2="17" y2="20" />
				<line x1="20" y1="14" x2="20" y2="20" />
				<line x1="17" y1="14" x2="20" y2="14" />
				<line x1="17" y1="17" x2="20" y2="17" />
			</svg>
		</Popover.Trigger>
		<Popover.Portal>
			<Popover.Content class="qr-popover" sideOffset={8} align="end">
				<div class="qr-header">
					<h4 class="qr-title">{m.share_room()} {code}</h4>
					<p class="qr-sub">{m.scan_to_join()}</p>
				</div>
				<div class="qr-code">
					{#if svg}
						<!-- eslint-disable-next-line svelte/no-at-html-tags -->
						{@html svg}
					{/if}
				</div>
				<div class="qr-url-row">
					<code class="qr-url">{url}</code>
					<button class="qr-copy" type="button" onclick={copy}>{m.copy_link()}</button>
				</div>
			</Popover.Content>
		</Popover.Portal>
	</Popover.Root>
{/if}

<style>
	:global(.qr-popover) {
		z-index: 60;
		width: 18rem;
		padding: var(--space-4);
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		background: var(--bg-surface-elevated);
		border: var(--border-width) var(--border-style) var(--border-color);
		border-radius: var(--radius-lg);
		box-shadow: var(--shadow-lg);
		animation: qr-in var(--duration-fast) var(--easing);
	}
	@keyframes qr-in {
		from {
			opacity: 0;
			transform: translateY(-4px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}
	.qr-header {
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
	}
	.qr-title {
		margin: 0;
		font-family: var(--font-heading);
		font-size: calc(1rem * var(--font-scale));
		color: var(--color-text);
	}
	.qr-sub {
		margin: 0;
		font-family: var(--font-body);
		font-size: calc(0.8rem * var(--font-scale));
		color: var(--color-text-muted);
	}
	.qr-code {
		display: flex;
		justify-content: center;
		padding: var(--space-3);
		background: #ffffff;
		border: var(--border-width) var(--border-style) var(--border-color);
		border-radius: var(--radius-md);
	}
	.qr-code :global(svg) {
		width: 12rem;
		height: 12rem;
		display: block;
	}
	.qr-url-row {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}
	.qr-url {
		flex: 1;
		min-width: 0;
		padding: var(--space-2) var(--space-3);
		background: var(--bg-surface);
		border: var(--border-width) var(--border-style) var(--border-color);
		border-radius: var(--radius-sm);
		font-family: var(--font-mono);
		font-size: calc(0.75rem * var(--font-scale));
		color: var(--color-text);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.qr-copy {
		flex-shrink: 0;
		padding: var(--space-2) var(--space-3);
		background: var(--color-primary);
		color: var(--color-text-inverse);
		border: none;
		border-radius: var(--radius-sm);
		font-family: var(--font-body);
		font-size: calc(0.8rem * var(--font-scale));
		font-weight: 600;
		cursor: pointer;
	}
	.qr-copy:hover {
		background: var(--color-primary-hover);
	}
</style>
