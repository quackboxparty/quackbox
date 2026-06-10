import * as Effect from 'effect/Effect';
import * as FileSystem from 'effect/FileSystem';
import * as Path from 'effect/Path';
import {
	type Pack,
	PackFile,
	PackOverlayFile,
	type PackOverlay,
	type Question,
	QuestionFile,
	QuestionOverlayFile,
	type QuestionOverlay,
	TagOverlayFiles,
	TagRegistryFiles
} from '../../schemas/index.ts';
import type { Tag, TagOverlay } from '../../schemas/index.ts';

import { DATA_DIR, TAG_CATEGORIES, type TagCategoryName } from './paths.ts';
import type { Entry, LoadedDataset, LoadIssue, LoadOptions, Overlays, Registry } from './shared.ts';
import { parse } from './util.ts';

export type { Entry, LoadedDataset, LoadIssue, LoadOptions, Overlays, Registry } from './shared.ts';
export { runCrossFileChecks } from './validate.ts';

/** All four loaders run sequentially inside one `Effect.gen`; the error
 *  channel is `never` because we accumulate load problems in the success
 *  value, per the data-layer "diagnostic accumulation" rule in AGENTS.md. */
export const loadDataset = (
	opts: LoadOptions = {}
): Effect.Effect<LoadedDataset, never, FileSystem.FileSystem | Path.Path> =>
	Effect.gen(function* () {
		const path = yield* Path.Path;
		const dataDir = opts.dataDir ?? DATA_DIR;
		const base = path.dirname(dataDir);
		const rel = (file: string) => path.relative(base, file);

		const questions = yield* loadQuestions(dataDir, rel);
		const packs = yield* loadPacks(dataDir, rel);
		const tags = yield* loadTags(dataDir, rel);
		const overlays = yield* loadOverlays(dataDir, rel);

		return {
			dataDir,
			issues: [...questions.issues, ...packs.issues, ...tags.issues, ...overlays.issues],
			overlays: overlays.items,
			packs: packs.items,
			questions: questions.items,
			tags: tags.items
		} satisfies LoadedDataset;
	});

function accumulateFileParses<A, R>(
	files: string[],
	processor: (file: string) => Effect.Effect<ReadonlyArray<A>, LoadIssue[], R>
): Effect.Effect<{ items: { file: string; item: A }[]; issues: LoadIssue[] }, never, R> {
	return Effect.gen(function* () {
		const results = yield* Effect.forEach(
			files,
			(file) =>
				processor(file).pipe(
					Effect.match({
						onSuccess: (items) => ({ ok: true as const, file, items }),
						onFailure: (errs) => ({ ok: false as const, file, errs })
					})
				),
			{ concurrency: 'unbounded' }
		);

		const issues: LoadIssue[] = [];
		const successes: { file: string; item: A }[] = [];

		for (const result of results) {
			if (!result.ok) {
				issues.push(...result.errs);
			} else {
				for (const item of result.items) {
					successes.push({ file: result.file, item });
				}
			}
		}

		return { items: successes, issues };
	});
}

function loadQuestions(
	dataDir: string,
	rel: (f: string) => string
): Effect.Effect<
	{ items: Registry<Question>; issues: LoadIssue[] },
	never,
	FileSystem.FileSystem | Path.Path
> {
	return Effect.gen(function* () {
		const path = yield* Path.Path;
		const questionsDir = path.join(dataDir, 'questions');
		const { files, issues: walkIssues } = yield* walkYaml(questionsDir);

		const { items: parsedQuestions, issues: parseIssues } = yield* accumulateFileParses(
			files,
			(file) => parse(file, QuestionFile)
		);

		const items = new Map<string, Entry<Question>>();
		const issues = [...walkIssues, ...parseIssues];

		for (const { file, item: q } of parsedQuestions) {
			if (items.has(q.id)) {
				issues.push({ file: rel(file), message: `duplicate question id found '${q.id}'` });
			} else {
				items.set(q.id, { file: rel(file), item: q });
			}
		}

		return { items, issues };
	});
}

function loadPacks(
	dataDir: string,
	rel: (f: string) => string
): Effect.Effect<
	{ items: Registry<Pack>; issues: LoadIssue[] },
	never,
	FileSystem.FileSystem | Path.Path
> {
	return Effect.gen(function* () {
		const path = yield* Path.Path;
		const packsDir = path.join(dataDir, 'packs');
		const { files, issues: walkIssues } = yield* walkYaml(packsDir);

		const { items: parsedPacks, issues: parseIssues } = yield* accumulateFileParses(
			files,
			(file) => parse(file, PackFile).pipe(Effect.map((pack) => [pack]))
		);

		const items = new Map<string, Entry<Pack>>();
		const issues = [...walkIssues, ...parseIssues];

		for (const { file, item: pack } of parsedPacks) {
			if (items.has(pack.id)) {
				issues.push({ file: rel(file), message: `duplicate pack id found '${pack.id}'` });
			} else {
				items.set(pack.id, { file: rel(file), item: pack });
			}
		}

		return { items, issues };
	});
}

function loadTags(
	dataDir: string,
	rel: (f: string) => string
): Effect.Effect<
	{ items: Registry<Tag>; issues: LoadIssue[] },
	never,
	FileSystem.FileSystem | Path.Path
> {
	return Effect.gen(function* () {
		const path = yield* Path.Path;
		const fs = yield* FileSystem.FileSystem;
		const tagsDir = path.join(dataDir, 'tags');

		const existingFiles: string[] = [];
		for (const category of TAG_CATEGORIES) {
			const file = path.join(tagsDir, `${category}.yaml`);
			if (yield* fs.exists(file).pipe(Effect.orElseSucceed(() => false))) existingFiles.push(file);
		}

		const { items: parsedTags, issues: parseIssues } = yield* accumulateFileParses(
			existingFiles,
			(file) => {
				const category = path.basename(file, '.yaml') as TagCategoryName;
				return parse(file, TagRegistryFiles[category]);
			}
		);

		const items = new Map<string, Entry<Tag>>();
		const issues = [...parseIssues];

		for (const { file, item: t } of parsedTags) {
			if (items.has(t.id)) {
				issues.push({ file: rel(file), message: `duplicate tag id found '${t.id}'` });
			} else {
				items.set(t.id, { file: rel(file), item: t });
			}
		}

		return { items, issues };
	});
}

function loadOverlays(
	dataDir: string,
	rel: (f: string) => string
): Effect.Effect<
	{ items: Overlays; issues: LoadIssue[] },
	never,
	FileSystem.FileSystem | Path.Path
> {
	return Effect.gen(function* () {
		const path = yield* Path.Path;
		const i18nDir = path.join(dataDir, 'i18n');
		const { files, issues: walkIssues } = yield* walkYaml(i18nDir);

		type OverlayEntry =
			| { locale: string; kind: 'questions'; filename: string; item: QuestionOverlay }
			| { locale: string; kind: 'packs'; filename: string; item: PackOverlay }
			| { locale: string; kind: 'tags'; filename: string; item: TagOverlay };

		const { items: parsedOverlays, issues: parseIssues } = yield* accumulateFileParses<
			OverlayEntry,
			FileSystem.FileSystem
		>(
			files,
			(file): Effect.Effect<OverlayEntry[], LoadIssue[], FileSystem.FileSystem> => {
				const r = path.relative(i18nDir, file);
				const parts = r.split(/[\\/]/);
				const locale = parts[0];
				const kind = parts[1];
				const filename = parts[2];

				if (!locale || !filename) {
					return Effect.fail([{ file, message: `invalid overlay path: ${r}` }]);
				}
				if (kind === 'questions') {
					return parse(file, QuestionOverlayFile).pipe(
						Effect.map((qs) => qs.map((q) => ({ locale, kind: 'questions' as const, filename, item: q })))
					);
				}
				if (kind === 'packs') {
					return parse(file, PackOverlayFile).pipe(
						Effect.map((p) => [{ locale, kind: 'packs' as const, filename, item: p }])
					);
				}
				if (kind === 'tags') {
					const category = filename.replace(/\.ya?ml$/, '') as TagCategoryName;
					if (!(TAG_CATEGORIES as readonly string[]).includes(category)) {
						return Effect.fail([{ file, message: `unknown tag category: ${category}` }]);
					}
					return parse(file, TagOverlayFiles[category]).pipe(
						Effect.map((ts) => ts.map((t) => ({ locale, kind: 'tags' as const, filename, item: t })))
					);
				}
				return Effect.fail([
					{ file, message: `unknown i18n subdirectory: ${kind ?? '(root)'}` }
				]);
			}
		);

		const overlays: Overlays = new Map();
		const issues = [...walkIssues, ...parseIssues];

		for (const { file, item: enriched } of parsedOverlays) {
			const localeOverlays = overlays.getOrInsertComputed(enriched.locale, () => ({
				packs: new Map(),
				questions: new Map(),
				tags: new Map()
			}));
			if (enriched.kind === 'questions') {
				localeOverlays.questions.set(enriched.item.id, { file: rel(file), item: enriched.item });
			} else if (enriched.kind === 'packs') {
				localeOverlays.packs.set(enriched.item.id, { file: rel(file), item: enriched.item });
			} else {
				localeOverlays.tags.set(enriched.item.id, { file: rel(file), item: enriched.item });
			}
		}

		return { items: overlays, issues };
	});
}

function walkYaml(
	dir: string
): Effect.Effect<
	{ files: string[]; issues: LoadIssue[] },
	never,
	FileSystem.FileSystem | Path.Path
> {
	return Effect.gen(function* () {
		const fs = yield* FileSystem.FileSystem;
		const path = yield* Path.Path;
		const files: string[] = [];
		const issues: LoadIssue[] = [];

		const exists = yield* fs.exists(dir).pipe(Effect.orElseSucceed(() => false));
		if (!exists) {
			return {
				files: [],
				issues: [{ file: dir, message: `dir doesn't exist or is inaccessible: ${dir}` }]
			};
		}
		const entries = yield* fs.readDirectory(dir).pipe(Effect.orElseSucceed(() => [] as string[]));

		for (const name of entries) {
			const full = path.join(dir, name);
			const stat = yield* fs.stat(full).pipe(Effect.option);
			if (stat._tag === 'None') continue;
			if (stat.value.type === 'Directory') {
				const sub = yield* walkYaml(full);
				files.push(...sub.files);
				issues.push(...sub.issues);
			} else if (stat.value.type === 'File' && /\.ya?ml$/.test(name)) {
				files.push(full);
			}
		}
		return { files, issues };
	});
}


