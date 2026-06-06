import { readdir, readFile, stat } from 'node:fs/promises';
import { dirname, extname, join, relative } from 'node:path';
import * as v from 'valibot';
import { parse as parseYaml } from 'yaml';

import {
	PackFile,
	PackOverlay,
	QuestionFile,
	QuestionOverlayFile,
	TagOverlayFiles,
	TagRegistryFiles,
	type Pack,
	type PackOverlay as PackOverlayT,
	type Question,
	type QuestionOverlay
} from '../schemas/index.ts';
import { DATA_DIR, TAG_CATEGORIES, type TagCategoryName } from './paths.ts';

export type LoadIssue = {
	file: string;
	path?: string;
	message: string;
};

export type LoadOptions = {
	dataDir?: string;
};

export type LoadedDataset = {
	questions: Array<{ file: string; item: Question }>;
	questionOverlays: Map<string, Array<{ file: string; locale: string; item: QuestionOverlay }>>;
	packs: Array<{ file: string; item: Pack }>;
	packOverlays: Map<string, Array<{ file: string; locale: string; item: PackOverlayT }>>;
	tagRegistry: Map<string, { file: string; defaultLang: string }>;
	issues: LoadIssue[];
	dataDir: string;
};

async function walkYaml(dir: string): Promise<string[]> {
	let entries;
	try {
		entries = await readdir(dir, { withFileTypes: true });
	} catch (err) {
		if ((err as NodeJS.ErrnoException).code === 'ENOENT') return [];
		throw err;
	}
	const out: string[] = [];
	for (const e of entries) {
		const full = join(dir, e.name);
		if (e.isDirectory()) out.push(...(await walkYaml(full)));
		else if (e.isFile() && (e.name.endsWith('.yaml') || e.name.endsWith('.yml'))) out.push(full);
	}
	return out;
}

function makeRel(dataDir: string) {
	const base = dirname(dataDir);
	return (file: string) => relative(base, file);
}

function formatIssuePath(issue: v.BaseIssue<unknown>): string {
	if (!issue.path) return '';
	return issue.path
		.map((p) => {
			const key = (p as { key?: unknown }).key;
			if (typeof key === 'number') return `[${key}]`;
			if (typeof key === 'string') return `.${key}`;
			return '';
		})
		.join('')
		.replace(/^\./, '');
}

function makePushIssues(rel: (f: string) => string) {
	return function pushIssues<T>(
		file: string,
		result: v.SafeParseResult<v.BaseSchema<unknown, T, v.BaseIssue<unknown>>>,
		out: LoadIssue[]
	): result is { success: true; output: T; typed: true; issues: undefined } {
		if (result.success) return true;
		for (const issue of result.issues) {
			out.push({ file: rel(file), path: formatIssuePath(issue), message: issue.message });
		}
		return false;
	};
}

async function readYamlFile(
	file: string,
	issues: LoadIssue[],
	rel: (f: string) => string,
	opts: { optional?: boolean } = {}
): Promise<unknown | undefined> {
	try {
		const text = await readFile(file, 'utf8');
		return parseYaml(text);
	} catch (err) {
		if (opts.optional && (err as NodeJS.ErrnoException).code === 'ENOENT') return undefined;
		issues.push({ file: rel(file), message: `failed to parse YAML: ${(err as Error).message}` });
		return undefined;
	}
}

export async function loadDataset(opts: LoadOptions = {}): Promise<LoadedDataset> {
	const dataDir = opts.dataDir ?? DATA_DIR;
	const questionsDir = join(dataDir, 'questions');
	const packsDir = join(dataDir, 'packs');
	const tagsDir = join(dataDir, 'tags');
	const i18nDir = join(dataDir, 'i18n');

	const rel = makeRel(dataDir);
	const pushIssues = makePushIssues(rel);
	const issues: LoadIssue[] = [];
	const ds: LoadedDataset = {
		questions: [],
		questionOverlays: new Map(),
		packs: [],
		packOverlays: new Map(),
		tagRegistry: new Map(),
		issues,
		dataDir
	};

	// canonical questions
	for (const file of await walkYaml(questionsDir)) {
		const raw = await readYamlFile(file, issues, rel);
		if (raw === undefined) continue;
		const result = v.safeParse(QuestionFile, raw);
		if (!pushIssues(file, result, issues)) continue;
		for (const item of result.output) ds.questions.push({ file: rel(file), item });
	}

	// canonical packs (single object per file)
	for (const file of await walkYaml(packsDir)) {
		const raw = await readYamlFile(file, issues, rel);
		if (raw === undefined) continue;
		const result = v.safeParse(PackFile, raw);
		if (!pushIssues(file, result, issues)) continue;
		ds.packs.push({ file: rel(file), item: result.output });
	}

	// tag registries — one file per category
	for (const category of TAG_CATEGORIES) {
		const file = join(tagsDir, `${category}.yaml`);
		const raw = await readYamlFile(file, issues, rel, { optional: true });
		if (raw === undefined) continue;
		const schema = TagRegistryFiles[category as TagCategoryName];
		const result = v.safeParse(schema, raw);
		if (!pushIssues(file, result, issues)) continue;
		for (const entry of result.output) {
			ds.tagRegistry.set(entry.id, { file: rel(file), defaultLang: entry.default_lang });
		}
	}

	// i18n overlays
	for (const file of await walkYaml(i18nDir)) {
		const r = relative(i18nDir, file);
		const parts = r.split(/[\\/]/);
		const locale = parts[0];
		const kind = parts[1];
		if (!locale) continue;
		const raw = await readYamlFile(file, issues, rel);
		if (raw === undefined) continue;

		if (kind === 'questions') {
			const result = v.safeParse(QuestionOverlayFile, raw);
			if (!pushIssues(file, result, issues)) continue;
			for (const item of result.output) {
				const list = ds.questionOverlays.get(item.id) ?? [];
				list.push({ file: rel(file), locale, item });
				ds.questionOverlays.set(item.id, list);
			}
		} else if (kind === 'packs') {
			const result = v.safeParse(PackOverlay, raw);
			if (!pushIssues(file, result, issues)) continue;
			const list = ds.packOverlays.get(result.output.id) ?? [];
			list.push({ file: rel(file), locale, item: result.output });
			ds.packOverlays.set(result.output.id, list);
		} else if (kind === 'tags') {
			const category = parts[2]?.replace(/\.ya?ml$/, '');
			if (!category || !(TAG_CATEGORIES as readonly string[]).includes(category)) {
				issues.push({
					file: rel(file),
					message: `tag overlay file must live at data/i18n/<locale>/tags/<category>.yaml; got category=${category}`
				});
				continue;
			}
			const schema = TagOverlayFiles[category as TagCategoryName];
			const result = v.safeParse(schema, raw);
			if (!pushIssues(file, result, issues)) continue;
		} else {
			issues.push({
				file: rel(file),
				message: `unknown i18n subdirectory: ${kind ?? '(root)'}`
			});
		}
	}

	return ds;
}

// ---------- cross-file checks ----------

const EXT_KIND_MAP: Record<string, 'image' | 'audio' | 'video'> = {
	'.png': 'image',
	'.jpg': 'image',
	'.jpeg': 'image',
	'.webp': 'image',
	'.svg': 'image',
	'.gif': 'image',
	'.avif': 'image',
	'.mp3': 'audio',
	'.ogg': 'audio',
	'.opus': 'audio',
	'.wav': 'audio',
	'.flac': 'audio',
	'.m4a': 'audio',
	'.mp4': 'video',
	'.webm': 'video',
	'.mov': 'video'
};

async function checkMediaFile(
	ref: string,
	kind: 'image' | 'audio' | 'video',
	contextFile: string,
	mediaDir: string,
	issues: LoadIssue[]
): Promise<void> {
	if (!ref.startsWith('media:')) return; // url:/youtube: are format-only
	const sub = ref.slice('media:'.length);
	const full = join(mediaDir, sub);
	let st;
	try {
		st = await stat(full);
	} catch {
		issues.push({ file: contextFile, message: `media file missing: ${ref}` });
		return;
	}

	const ext = extname(sub).toLowerCase();
	const actual = EXT_KIND_MAP[ext];
	if (!actual) {
		issues.push({ file: contextFile, message: `unknown media extension: ${ext} (${ref})` });
	} else {
		// A video file may be used as audio-only (play track, hide visuals).
		// All other ext to kind mismatches are an error.
		const ok = actual === kind || (actual === 'video' && kind === 'audio');
		if (!ok) {
			issues.push({
				file: contextFile,
				message: `media kind mismatch: declared ${kind} but extension ${ext} is ${actual} (${ref})`
			});
		}
	}

	const KB = 1024;
	const MB = 1024 * KB;
	const cap = kind === 'image' ? 100 * KB : 1 * MB;
	if (st.size > cap) {
		issues.push({
			file: contextFile,
			message: `media file exceeds size cap (${st.size}B > ${cap}B): ${ref}`
		});
	}
}

function collectQuestionMedia(
	q: Question
): Array<{ kind: 'image' | 'audio' | 'video'; ref: string }> {
	const out: Array<{ kind: 'image' | 'audio' | 'video'; ref: string }> = [];
	for (const m of q.content.prompt.media ?? []) out.push({ kind: m.kind, ref: m.ref });
	if (q.kind === 'order') {
		for (const it of q.content.items)
			for (const m of it.media ?? []) out.push({ kind: m.kind, ref: m.ref });
	} else {
		const mc = q.content.variants.multiple_choice;
		if (mc)
			for (const c of mc.choices)
				for (const m of c.media ?? []) out.push({ kind: m.kind, ref: m.ref });
	}
	return out;
}

export async function runCrossFileChecks(ds: LoadedDataset): Promise<void> {
	const { issues } = ds;
	const mediaDir = join(ds.dataDir, 'media');

	// 1. duplicate question ids
	const seenQ = new Map<string, string>();
	for (const { file, item } of ds.questions) {
		const prev = seenQ.get(item.id);
		if (prev) {
			issues.push({ file, message: `duplicate question id ${item.id} (also in ${prev})` });
		} else {
			seenQ.set(item.id, file);
		}
	}

	// 2. duplicate pack ids
	const seenP = new Map<string, string>();
	for (const { file, item } of ds.packs) {
		const prev = seenP.get(item.id);
		if (prev) {
			issues.push({ file, message: `duplicate pack id ${item.id} (also in ${prev})` });
		} else {
			seenP.set(item.id, file);
		}
	}

	// 3. tag refs from questions + pack filters must exist in registry
	for (const { file, item } of ds.questions) {
		for (const t of item.tags) {
			if (!ds.tagRegistry.has(t)) {
				issues.push({ file, message: `unknown tag ${t} on question ${item.id}` });
			}
		}
	}
	for (const { file, item } of ds.packs) {
		const f = item.filter;
		const allTags = [...(f?.tags_all ?? []), ...(f?.tags_any ?? []), ...(f?.tags_none ?? [])];
		for (const t of allTags) {
			if (!ds.tagRegistry.has(t)) {
				issues.push({ file, message: `unknown tag ${t} on pack ${item.id}` });
			}
		}
	}

	// 4. pack questions[] + includes[] must resolve; replaced_by must resolve
	for (const { file, item } of ds.packs) {
		for (const qid of item.questions ?? []) {
			if (!seenQ.has(qid)) {
				issues.push({ file, message: `pack ${item.id} references unknown question ${qid}` });
			}
		}
		for (const pid of item.includes ?? []) {
			if (!seenP.has(pid)) {
				issues.push({ file, message: `pack ${item.id} includes unknown pack ${pid}` });
			}
		}
	}
	for (const { file, item } of ds.questions) {
		const dep = item.deprecated;
		if (dep?.replaced_by && !seenQ.has(dep.replaced_by)) {
			issues.push({
				file,
				message: `question ${item.id} replaced_by unknown question ${dep.replaced_by}`
			});
		}
	}

	// 5. overlay ids must resolve to a canonical question/pack
	for (const [qid, list] of ds.questionOverlays) {
		if (!seenQ.has(qid)) {
			for (const o of list) {
				issues.push({ file: o.file, message: `overlay references unknown question ${qid}` });
			}
		}
	}
	for (const [pid, list] of ds.packOverlays) {
		if (!seenP.has(pid)) {
			for (const o of list) {
				issues.push({ file: o.file, message: `overlay references unknown pack ${pid}` });
			}
		}
	}

	// 6. pack `includes` cycle detection
	const packGraph = new Map<string, string[]>();
	for (const { item } of ds.packs) packGraph.set(item.id, item.includes ?? []);
	const fileById = new Map<string, string>();
	for (const { file, item } of ds.packs) fileById.set(item.id, file);
	const state = new Map<string, 0 | 1 | 2>();
	function dfs(node: string, stack: string[]): void {
		const s = state.get(node);
		if (s === 2) return;
		if (s === 1) {
			const cycle = [...stack.slice(stack.indexOf(node)), node].join(' -> ');
			issues.push({
				file: fileById.get(node) ?? '(unknown)',
				message: `pack includes cycle: ${cycle}`
			});
			return;
		}
		state.set(node, 1);
		for (const next of packGraph.get(node) ?? []) {
			if (packGraph.has(next)) dfs(next, [...stack, node]);
		}
		state.set(node, 2);
	}
	for (const id of packGraph.keys()) dfs(id, []);

	// 7. media file existence + extension + size
	for (const { file, item } of ds.questions) {
		for (const m of collectQuestionMedia(item)) {
			await checkMediaFile(m.ref, m.kind, file, mediaDir, issues);
		}
	}
}
