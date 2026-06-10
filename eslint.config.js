import js from '@eslint/js';
import prettier from 'eslint-config-prettier';
import svelte from 'eslint-plugin-svelte';
import { defineConfig } from 'eslint/config';
import globals from 'globals';
import ts from 'typescript-eslint';
import effectPlugin from '@effect/eslint-plugin';

import svelteConfig from './svelte.config.js';

export default defineConfig(
	// global ignores — must be the sole key for flat-config global ignore semantics
	{
		ignores: [
			'.svelte-kit/**',
			'build/**',
			'dist/**',
			'node_modules/**',
			'src/lib/paraglide/**',
			'coverage/**',
			'playwright-report/**',
			'test-results/**',
			'schemas/**'
		]
	},

	js.configs.recommended,
	ts.configs.strictTypeChecked,
	ts.configs.stylisticTypeChecked,
	svelte.configs.recommended,
	prettier,
	svelte.configs.prettier,

	{
		languageOptions: {
			globals: { ...globals.browser, ...globals.node },
			parserOptions: {
				extraFileExtensions: ['.svelte'],
				projectService: true,
				tsconfigRootDir: import.meta.dirname
			}
		},
		rules: {
			'@typescript-eslint/consistent-type-imports': [
				'error',
				{ fixStyle: 'inline-type-imports', prefer: 'type-imports' }
			],

			'@typescript-eslint/no-misused-promises': [
				'error',
				{ checksVoidReturn: { attributes: false } }
			],
			'@typescript-eslint/no-unused-vars': [
				'error',
				{
					argsIgnorePattern: '^_',
					caughtErrorsIgnorePattern: '^_',
					destructuredArrayIgnorePattern: '^_',
					varsIgnorePattern: '^_'
				}
			],
			'@typescript-eslint/restrict-template-expressions': [
				'error',
				{ allowBoolean: true, allowNumber: true }
			],
			'no-undef': 'off'
		}
	},

	{
		files: ['**/*.svelte', '**/*.svelte.ts', '**/*.svelte.js'],
		languageOptions: {
			parserOptions: {
				extraFileExtensions: ['.svelte'],
				parser: ts.parser,
				projectService: true,
				svelteConfig,
				tsconfigRootDir: import.meta.dirname
			}
		},
		rules: {
			// runes + $props/$state often trigger these falsely
			'@typescript-eslint/no-redeclare': 'off',
			'@typescript-eslint/no-unused-expressions': 'off'
		}
	},

	{
		files: ['*.config.{js,ts}', 'scripts/**/*.{js,ts}'],
		languageOptions: {
			globals: { ...globals.node }
		}
	},

	{
		extends: [ts.configs.disableTypeChecked],
		files: ['*.js', '*.config.js', '*.config.ts']
	},

	{
		files: ['**/*.{test,spec}.{js,ts,svelte}', 'e2e/**/*.{js,ts}'],
		rules: {
			'@typescript-eslint/no-explicit-any': 'off',
			'@typescript-eslint/no-non-null-assertion': 'off'
		}
	},

	// Force specific submodule imports over the `effect` barrel, mirroring the
	// Effect language service's `importFromBarrel` diagnostic. Keeps tree-shaking
	// honest and matches the v4 split-package convention.
	//
	// Exception: the `src/lib/schemas/` directory is a leaf that genuinely
	// needs the whole Schema namespace — `effect/Schema` doesn't re-export
	// one, and listing every individual import would obscure the schema
	// definitions. We rely on the `import type` rule (above) to keep the
	// client bundle clean and on SvelteKit's `$lib/server/` convention for
	// the runtime values.
	{
		plugins: { '@effect': effectPlugin },
		rules: {
			'@effect/no-import-from-barrel-package': ['error', { packageNames: ['effect'] }]
		},
		ignores: ['src/lib/schemas/**']
	}
);
