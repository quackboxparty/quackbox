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

function systemPrefersDark(): boolean {
	if (!browser) return false;
	return window.matchMedia('(prefers-color-scheme: dark)').matches;
}

export function getTheme(): ThemeId {
	if (!browser) return 'modern';
	const stored = localStorage.getItem(STORAGE_KEY) as ThemeId | null;
	if (stored && stored in themes) return stored;
	return systemPrefersDark() ? 'modern-dark' : 'modern';
}

export function setTheme(id: ThemeId): void {
	if (!browser) return;
	document.documentElement.setAttribute('data-theme', id === 'modern' ? '' : id);
	localStorage.setItem(STORAGE_KEY, id);
}

export function setSystemTheme(): void {
	if (!browser) return;
	localStorage.removeItem(STORAGE_KEY);
	const theme = systemPrefersDark() ? 'modern-dark' : 'modern';
	document.documentElement.setAttribute('data-theme', theme === 'modern' ? '' : theme);
}

export function hasStoredTheme(): boolean {
	if (!browser) return false;
	return localStorage.getItem(STORAGE_KEY) !== null;
}

export function initTheme(): void {
	if (!browser) return;
	const theme = getTheme();
	document.documentElement.setAttribute('data-theme', theme === 'modern' ? '' : theme);
}
