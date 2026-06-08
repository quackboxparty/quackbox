import { paraglideVitePlugin } from '@inlang/paraglide-js';
import { sveltekit } from '@sveltejs/kit/vite';
import { playwright } from '@vitest/browser-playwright';
import { defineConfig } from 'vitest/config';

export default defineConfig({
	plugins: [
		sveltekit(),
		paraglideVitePlugin({ outdir: './src/lib/paraglide', project: './project.inlang' })
	],
	test: {
		expect: { requireAssertions: true },
		projects: [
			{
				extends: './vite.config.ts',
				test: {
					browser: {
						enabled: true,
						instances: [{ browser: 'chromium', headless: true }],
						provider: playwright()
					},
					exclude: ['src/lib/server/**'],
					include: ['src/**/*.svelte.{test,spec}.{js,ts}'],
					name: 'client'
				}
			},

			{
				extends: './vite.config.ts',
				test: {
					environment: 'node',
					exclude: ['src/**/*.svelte.{test,spec}.{js,ts}'],
					include: ['src/**/*.{test,spec}.{js,ts}'],
					name: 'server'
				}
			}
		]
	}
});
