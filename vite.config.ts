import { paraglideVitePlugin } from '@inlang/paraglide-js';
import { sveltekit } from '@sveltejs/kit/vite';
import { playwright } from '@vitest/browser-playwright';
import { defineConfig } from 'vitest/config';

export default defineConfig({
  server: {
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:3000'
      },
      '/media': {
        target: 'http://127.0.0.1:3000'
      },
      '/ws': {
        target: 'http://127.0.0.1:3000',
        changeOrigin: true,
        ws: true
      }
    }
  },
  plugins: [
    sveltekit(),
    paraglideVitePlugin({
      outdir: './src/lib/paraglide',
      project: './project.inlang',
      strategy: ['cookie', 'preferredLanguage', 'baseLocale']
    })
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
