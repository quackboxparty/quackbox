import { browser } from '$app/environment';

export type ThemeId =
	| 'modern'
	| 'modern-dark'
	| 'retro'
	| 'medieval'
	| 'neon'
	| 'chalkboard'
	| 'kawaii'
	| 'western'
	| 'wizard';

export interface ThemeMeta {
	id: ThemeId;
	label: string;
	/** Emojis floating in the background */
	emojis: string[];
}

export const themes: Record<ThemeId, ThemeMeta> = {
	modern: {
		id: 'modern',
		label: 'Modern',
		emojis: ['🦆', '❓', '💡', '🧠', '🎯', '✨', '📚', '🏆']
	},
	'modern-dark': {
		id: 'modern-dark',
		label: 'Modern Dark',
		emojis: ['🌙', '🦆', '❓', '💡', '🧠', '🎯', '✨', '🔭']
	},
	retro: {
		id: 'retro',
		label: 'Retro',
		emojis: ['👾', '🕹️', '👾', '⬛', '🟦', '🍄', '⭐', '🟡']
	},
	medieval: {
		id: 'medieval',
		label: 'Medieval',
		emojis: ['⚔️', '🛡️', '🏰', '🐉', '👑', '📜', '🏹', '🔮']
	},
	neon: {
		id: 'neon',
		label: 'Neon',
		emojis: ['🌆', '⚡', '💊', '🔫', '🤖', '🧬', '💿', '🔮']
	},
	chalkboard: {
		id: 'chalkboard',
		label: 'Chalkboard',
		emojis: ['✏️', '📐', '📖', '🧪', '🌍', '🗑️', '🎒', '🔬']
	},
	kawaii: {
		id: 'kawaii',
		label: 'Kawaii',
		emojis: ['🌸', '🦋', '🌈', '🐱', '🧁', '🎀', '☁️', '💕']
	},
	western: {
		id: 'western',
		label: 'Western',
		emojis: ['🤠', '🐎', '🌵', '🔫', '💰', '🏜️', '🐄', '🎻']
	},
	wizard: {
		id: 'wizard',
		label: 'Wizard',
		emojis: ['🧙', '🔮', '✨', '📖', '⚡', '🦉', '⚗️', '🪄']
	}
};

const STORAGE_KEY = 'quackbox-theme';

/** Detect system color scheme preference */
function systemPrefersDark(): boolean {
	if (!browser) return false;
	return window.matchMedia('(prefers-color-scheme: dark)').matches;
}

/** Resolve theme: stored preference → system preference → modern fallback */
export function getTheme(): ThemeId {
	if (!browser) return 'modern';
	const stored = localStorage.getItem(STORAGE_KEY) as ThemeId | null;
	if (stored && stored in themes) return stored;
	return systemPrefersDark() ? 'modern-dark' : 'modern';
}

/** Apply a theme: sets data-theme attr, persists choice */
export function setTheme(id: ThemeId): void {
	if (!browser) return;
	document.documentElement.setAttribute('data-theme', id === 'modern' ? '' : id);
	localStorage.setItem(STORAGE_KEY, id);
}

/** Clear stored preference and revert to system preference */
export function setSystemTheme(): void {
	if (!browser) return;
	localStorage.removeItem(STORAGE_KEY);
	const theme = systemPrefersDark() ? 'modern-dark' : 'modern';
	document.documentElement.setAttribute('data-theme', theme === 'modern' ? '' : theme);
}

/** Whether user has an explicit theme stored */
export function hasStoredTheme(): boolean {
	if (!browser) return false;
	return localStorage.getItem(STORAGE_KEY) !== null;
}

/** Initialize theme on app boot — call once from a top-level +layout.svelte */
export function initTheme(): void {
	if (!browser) return;
	const theme = getTheme();
	document.documentElement.setAttribute('data-theme', theme === 'modern' ? '' : theme);
}
