import {
	overwriteGetLocale,
	overwriteSetLocale,
	getLocale as baseGetLocale,
	setLocale as baseSetLocale,
	type Locale
} from '$lib/paraglide/runtime';
import { m } from '$lib/paraglide/messages';
import type { ThemeId } from '$lib/themes';

const themeLabels: Record<ThemeId, () => string> = {
	modern: m.theme_modern,
	'modern-dark': m.theme_modern_dark,
	retro: m.theme_retro,
	medieval: m.theme_medieval,
	neon: m.theme_neon,
	chalkboard: m.theme_chalkboard,
	kawaii: m.theme_kawaii,
	western: m.theme_western,
	wizard: m.theme_wizard
};

export function themeLabel(id: ThemeId): string {
	return themeLabels[id]();
}

/**
 * Paraglide's `m.*()` read `getLocale()` internally but return plain strings,
 * so Svelte can't see the locale dependency and never re-renders on switch.
 * Backing the locale with a `$state` rune and routing `getLocale` through it
 * makes every message call reactive — switching updates the UI without reload.
 */
const locale = $state({ current: baseGetLocale() });

export function initReactiveLocale(): void {
	// baseSetLocale is an ES live binding that overwriteSetLocale reassigns.
	// Capture the original before overwriting, or the override calls itself.
	const originalSetLocale = baseSetLocale;
	overwriteGetLocale(() => locale.current);
	overwriteSetLocale((next: Locale) => {
		void originalSetLocale(next, { reload: false });
		locale.current = next;
	});
}

export function currentLocale(): Locale {
	return locale.current;
}
