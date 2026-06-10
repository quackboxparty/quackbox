import * as Effect from 'effect/Effect';
import * as FileSystem from 'effect/FileSystem';
import * as Path from 'effect/Path';
import {
	type Pack,
	PackFile,
	PackOverlayFile,
	type Question,
	QuestionFile,
	QuestionOverlayFile,
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
		const items = new Map<string, Entry<Question>>();
		const issues: LoadIssue[] = [];

		const questionsDir = path.join(dataDir, 'questions');
		const { files, issues: walkIssues } = yield* walkYaml(questionsDir);
		issues.push(...walkIssues);

		yield* Effect.forEach(
			files,
			(file) =>
				parse(file, QuestionFile).pipe(
					Effect.match({
						onSuccess: (questions) => {
							for (const q of questions) {
								if (items.has(q.id)) {
									issues.push({ file, message: `duplicate question id found '${q.id}'` });
								} else {
									items.set(q.id, { file: rel(file), item: q });
								}
							}
						},
						onFailure: (errs) => {
							issues.push(...errs);
						}
					})
				),
			{ concurrency: 'unbounded' }
		);

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
		const items = new Map<string, Entry<Pack>>();
		const issues: LoadIssue[] = [];

		const packsDir = path.join(dataDir, 'packs');
		const { files, issues: walkIssues } = yield* walkYaml(packsDir);
		issues.push(...walkIssues);

		yield* Effect.forEach(
			files,
			(file) =>
				parse(file, PackFile).pipe(
					Effect.match({
						onSuccess: (pack) => {
							if (items.has(pack.id)) {
								issues.push({ file, message: `duplicate pack id found '${pack.id}'` });
							} else {
								items.set(pack.id, { file: rel(file), item: pack });
							}
						},
						onFailure: (errs) => {
							issues.push(...errs);
						}
					})
				),
			{ concurrency: 'unbounded' }
		);

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
		const items = new Map<string, Entry<Tag>>();
		const issues: LoadIssue[] = [];

		const tagsDir = path.join(dataDir, 'tags');
		yield* Effect.forEach(
			TAG_CATEGORIES,
			(category) =>
				Effect.gen(function* () {
					const file = path.join(tagsDir, `${category}.yaml`);
					const exists = yield* fs.exists(file).pipe(Effect.orElseSucceed(() => false));
					if (!exists) return;
					yield* parse(file, TagRegistryFiles[category]).pipe(
						Effect.match({
							onSuccess: (tags) => {
								for (const t of tags) {
									if (items.has(t.id)) {
										issues.push({ file, message: `duplicate tag id found '${t.id}'` });
									} else {
										items.set(t.id, { file: rel(file), item: t });
									}
								}
							},
							onFailure: (errs) => {
								issues.push(...errs);
							}
						})
					);
				}),
			{ concurrency: 'unbounded' }
		);

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
		const overlays: Overlays = new Map();
		const issues: LoadIssue[] = [];

		const i18nDir = path.join(dataDir, 'i18n');
		const { files, issues: walkIssues } = yield* walkYaml(i18nDir);
		issues.push(...walkIssues);

		yield* Effect.forEach(
			files,
			(file) =>
				Effect.gen(function* () {
					const r = path.relative(i18nDir, file);
					const parts = r.split(/[\\/]/);
					const locale = parts[0];
					const kind = parts[1];
					const filename = parts[2];
					if (!locale || !filename) return;

					let localeOverlays = overlays.get(locale);
					if (!localeOverlays) {
						localeOverlays = { packs: new Map(), questions: new Map(), tags: new Map() };
						overlays.set(locale, localeOverlays);
					}

					if (kind === 'questions') {
						yield* parse(file, QuestionOverlayFile).pipe(
							Effect.match({
								onSuccess: (questions) => {
									for (const q of questions) {
										localeOverlays.questions.set(q.id, { file: rel(file), item: q });
									}
								},
								onFailure: (errs) => {
									issues.push(...errs);
								}
							})
						);
					} else if (kind === 'packs') {
						yield* parse(file, PackOverlayFile).pipe(
							Effect.match({
								onSuccess: (pack) => {
									localeOverlays.packs.set(pack.id, { file: rel(file), item: pack });
								},
								onFailure: (errs) => {
									issues.push(...errs);
								}
							})
						);
					} else if (kind === 'tags') {
						const { items, issues: tagIssues } = yield* loadTagOverlay(file, filename, rel);
						localeOverlays.tags = new Map([...localeOverlays.tags, ...items]);
						issues.push(...tagIssues);
					} else {
						issues.push({
							file: rel(file),
							message: `unknown i18n subdirectory: ${kind ?? '(root)'}`
						});
					}
				}),
			{ concurrency: 'unbounded' }
		);

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

function loadTagOverlay(
	file: string,
	filename: string,
	rel: (f: string) => string
): Effect.Effect<
	{ items: Registry<TagOverlay>; issues: LoadIssue[] },
	never,
	FileSystem.FileSystem
> {
	return Effect.gen(function* () {
		const tagOverlays = new Map<string, Entry<TagOverlay>>();
		const issues: LoadIssue[] = [];

		const category = filename.replace(/\.ya?ml$/, '');
		if (!category || !(TAG_CATEGORIES as readonly string[]).includes(category)) {
			issues.push({
				file: rel(file),
				message: `tag overlay file must live at data/i18n/<locale>/tags/<category>.yaml; got category=${category}`
			});
			return { items: tagOverlays, issues };
		}
		yield* parse(file, TagOverlayFiles[category as TagCategoryName]).pipe(
			Effect.match({
				onSuccess: (tags) => {
					for (const t of tags) {
						tagOverlays.set(t.id, { file: rel(file), item: t });
					}
				},
				onFailure: (errs) => {
					issues.push(...errs);
				}
			})
		);

		return { items: tagOverlays, issues };
	});
}
