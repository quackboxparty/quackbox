<script lang="ts">
	import ThemeSwitcher from '$lib/components/ThemeSwitcher.svelte';
</script>

<svelte:head>
	<title>Quackbox</title>
</svelte:head>

<div class="page">
	<h1>Quackbox</h1>
	<p>Multi-gamemode quiz platform</p>

	<section class="demo">
		<h2>Theme</h2>
		<ThemeSwitcher />
	</section>

	<section class="demo">
		<h2>Token preview</h2>
		<div class="card-grid">
			<div class="card">
				<h3>Surface</h3>
				<p>bg-surface + shadow-md</p>
			</div>
			<div class="card">
				<h3>Elevated</h3>
				<p>bg-surface-elevated + shadow-lg</p>
			</div>
			<div class="card accent">
				<h3>Primary</h3>
				<p>color-primary background</p>
			</div>
		</div>
	</section>

	<section class="demo">
		<h2>Radius scale</h2>
		<div class="radius-demo">
			<div class="radius-box" style="border-radius: var(--radius-none)">none</div>
			<div class="radius-box" style="border-radius: var(--radius-sm)">sm</div>
			<div class="radius-box" style="border-radius: var(--radius-md)">md</div>
			<div class="radius-box" style="border-radius: var(--radius-lg)">lg</div>
			<div class="radius-box" style="border-radius: var(--radius-full)">full</div>
		</div>
	</section>

	<section class="demo">
		<h2>Colors</h2>
		<div class="color-row">
			<span class="swatch" style="background: var(--color-primary)"></span> primary
			<span class="swatch" style="background: var(--color-secondary)"></span> secondary
			<span class="swatch" style="background: var(--color-accent)"></span> accent
			<span class="swatch" style="background: var(--color-correct); border-color: var(--color-correct)"></span> correct
			<span class="swatch" style="background: var(--color-wrong); border-color: var(--color-wrong)"></span> wrong
		</div>
	</section>

	<section class="demo">
		<h2>Border width</h2>
		<div class="border-demo">
			<div class="border-box">border-width in action</div>
		</div>
	</section>

	<section class="demo">
		<h2>Shadow scale</h2>
		<div class="shadow-demo">
			<div class="shadow-box" style="box-shadow: var(--shadow-sm)">sm</div>
			<div class="shadow-box" style="box-shadow: var(--shadow-md)">md</div>
			<div class="shadow-box" style="box-shadow: var(--shadow-lg)">lg</div>
		</div>
	</section>
</div>

<style>
	.page {
		max-width: 48rem;
		margin: 0 auto;
		padding: var(--space-8);
		color: var(--color-text);
		font-family: var(--font-body);
		line-height: var(--line-height);
	}

	h1,
	h2,
	h3 {
		font-family: var(--font-heading);
		font-size: calc(2rem * var(--font-scale));
		color: var(--color-text);
		margin: 0 0 var(--space-4) 0;
	}

	h2 {
		font-size: calc(1.25rem * var(--font-scale));
	}

	h3 {
		font-size: calc(1rem * var(--font-scale));
	}

	p {
		margin: 0 0 var(--space-4) 0;
		color: var(--color-text-muted);
	}

	.demo {
		margin-bottom: var(--space-8);
	}

	.card-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(12rem, 1fr));
		gap: var(--space-4);
	}

	.card {
		background: var(--bg-surface);
		border: var(--border-width) var(--border-style) var(--border-color);
		border-radius: var(--radius-md);
		box-shadow: var(--shadow-md);
		padding: var(--space-6);
	}

	.card.accent {
		background: var(--color-primary);
		border-color: var(--color-primary);
		color: var(--color-text-inverse);
	}

	.card.accent h3,
	.card.accent p {
		color: var(--color-text-inverse);
	}

	.radius-demo {
		display: flex;
		gap: var(--space-4);
		flex-wrap: wrap;
	}

	.radius-box {
		width: 4rem;
		height: 4rem;
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--bg-surface);
		border: var(--border-width) var(--border-style) var(--border-color);
		font-size: calc(0.75rem * var(--font-scale));
		color: var(--color-text-muted);
	}

	.color-row {
		display: flex;
		gap: var(--space-4);
		align-items: center;
		flex-wrap: wrap;
		font-family: var(--font-mono);
		font-size: calc(0.875rem * var(--font-scale));
	}

	.swatch {
		display: inline-block;
		width: 1.5rem;
		height: 1.5rem;
		border-radius: var(--radius-sm);
		border: var(--border-width) var(--border-style) var(--border-color);
		vertical-align: middle;
		margin-right: var(--space-1);
	}

	.border-demo {
		display: flex;
	}

	.border-box {
		flex: 1;
		padding: var(--space-6);
		border: var(--border-width) var(--border-style) var(--border-color);
		border-radius: var(--radius-md);
		background: var(--bg-surface);
		text-align: center;
		color: var(--color-text-muted);
	}

	.shadow-demo {
		display: flex;
		gap: var(--space-6);
		flex-wrap: wrap;
	}

	.shadow-box {
		width: 6rem;
		height: 6rem;
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--bg-surface);
		border-radius: var(--radius-md);
		border: var(--border-width) var(--border-style) var(--border-color);
		font-size: calc(0.875rem * var(--font-scale));
		color: var(--color-text-muted);
	}
</style>
