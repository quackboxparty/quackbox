<script lang="ts">
	import { Label } from 'bits-ui';

	let {
		value = $bindable(''),
		placeholder = '',
		label = '',
		type = 'text',
		required = false
	}: {
		value?: string;
		placeholder?: string;
		label?: string;
		type?: string;
		required?: boolean;
	} = $props();
</script>

<div class="text-input-group">
	{#if label}
		<Label.Root class="text-input-label">{label}</Label.Root>
	{/if}
	<input {type} bind:value {placeholder} {required} class="text-input" />
</div>

<style>
	:global(.text-input-group) {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}
	:global(.text-input-label) {
		font-family: var(--font-heading);
		font-size: 0.875rem;
		color: var(--color-text);
	}
	:global(.text-input) {
		font-family: var(--font-body);
		font-size: calc(1rem * var(--font-scale));
		padding: var(--space-2) var(--space-3);
		color: var(--color-text);
		background: var(--bg-surface);
		border: var(--border-width) var(--border-style) var(--border-color);
		border-radius: var(--radius-md);
		transition: border-color var(--duration-fast) var(--easing);
		outline: none;
	}
	:global(.text-input)::placeholder {
		color: var(--color-text-muted);
	}
	:global(.text-input:focus) {
		border-color: var(--color-primary);
		box-shadow: var(--focus-ring);
	}
</style>
