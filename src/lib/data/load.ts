import type { Dirent } from 'node:fs';

import { readdir, readFile } from 'node:fs/promises';
import { dirname, join, relative } from 'node:path';
import * as v from 'valibot';
import { parse as parseYaml } from 'yaml';

import {
  type Pack,
  PackFile,
  PackOverlay,
  type PackOverlay as PackOverlayT,
  type Question,
  QuestionFile,
  type QuestionOverlay,
  QuestionOverlayFile,
  TagOverlayFiles,
  TagRegistryFiles
} from '../schemas/index.ts';
import { DATA_DIR, TAG_CATEGORIES, type TagCategoryName } from './paths.ts';

export { runCrossFileChecks } from './validate.ts';

export interface LoadedDataset {
  dataDir: string;
  issues: LoadIssue[];
  packOverlays: Map<string, { file: string; item: PackOverlayT; locale: string }[]>;
  packs: { file: string; item: Pack }[];
  questionOverlays: Map<string, { file: string; item: QuestionOverlay; locale: string }[]>;
  questions: { file: string; item: Question }[];
  tagRegistry: Map<string, { defaultLang: string; file: string }>;
}

export interface LoadIssue {
  file: string;
  message: string;
  path?: string;
}

export interface LoadOptions {
  dataDir?: string;
}

export async function loadDataset(opts: LoadOptions = {}): Promise<LoadedDataset> {
  const dataDir = opts.dataDir ?? DATA_DIR;
  const base = dirname(dataDir);
  const rel = (file: string) => relative(base, file);
  const issues: LoadIssue[] = [];

  const ds: LoadedDataset = {
    dataDir,
    issues,
    packOverlays: new Map(),
    packs: [],
    questionOverlays: new Map(),
    questions: [],
    tagRegistry: new Map()
  };

  await loadQuestions(ds, dataDir, rel);
  await loadPacks(ds, dataDir, rel);
  await loadTagRegistries(ds, dataDir, rel);
  await loadOverlays(ds, dataDir, rel);

  return ds;
}

async function loadQuestions(
  ds: LoadedDataset,
  dataDir: string,
  rel: (f: string) => string
): Promise<void> {
  const questionsDir = join(dataDir, 'questions');
  for (const file of await walkYaml(questionsDir)) {
    const raw = await readYamlFile(file, ds.issues, rel);
    if (raw === undefined) continue;
    const result = v.safeParse(QuestionFile, raw);
    if (!result.success) {
      pushIssues(rel(file), result.issues, ds.issues);
      continue;
    }
    for (const item of result.output) ds.questions.push({ file: rel(file), item });
  }
}

async function loadPacks(
  ds: LoadedDataset,
  dataDir: string,
  rel: (f: string) => string
): Promise<void> {
  const packsDir = join(dataDir, 'packs');
  for (const file of await walkYaml(packsDir)) {
    const raw = await readYamlFile(file, ds.issues, rel);
    if (raw === undefined) continue;
    const result = v.safeParse(PackFile, raw);
    if (!result.success) {
      pushIssues(rel(file), result.issues, ds.issues)
      continue;
    }
    ds.packs.push({ file: rel(file), item: result.output });
  }
}

async function loadOverlays(
  ds: LoadedDataset,
  dataDir: string,
  rel: (f: string) => string
): Promise<void> {
  const i18nDir = join(dataDir, 'i18n');
  for (const file of await walkYaml(i18nDir)) {
    const r = relative(i18nDir, file);
    const parts = r.split(/[\\/]/);
    const locale = parts[0];
    const kind = parts[1];
    if (!locale) continue;

    const raw = await readYamlFile(file, ds.issues, rel);
    if (raw === undefined) continue;

    if (kind === 'questions') {
      const result = v.safeParse(QuestionOverlayFile, raw);
      if (!result.success) {
        pushIssues(rel(file), result.issues, ds.issues);
        continue;
      }
      for (const item of result.output) {
        const list = ds.questionOverlays.get(item.id) ?? [];
        list.push({ file: rel(file), item, locale });
        ds.questionOverlays.set(item.id, list);
      }
    } else if (kind === 'packs') {
      const result = v.safeParse(PackOverlay, raw);
      if (!result.success) {
        pushIssues(rel(file), result.issues, ds.issues)
        continue;
      }
      const list = ds.packOverlays.get(result.output.id) ?? [];
      list.push({ file: rel(file), item: result.output, locale });
      ds.packOverlays.set(result.output.id, list);
    } else if (kind === 'tags') {
      await loadTagOverlay(ds, file, parts, rel);
    } else {
      ds.issues.push({
        file: rel(file),
        message: `unknown i18n subdirectory: ${kind ?? '(root)'}`
      });
    }
  }
}

async function walkYaml(dir: string): Promise<string[]> {
  let entries: Dirent[];
  try {
    entries = await readdir(dir, { withFileTypes: true });
  } catch (err) {
    if ((err as NodeJS.ErrnoException).code === 'ENOENT') return [];
    throw err;
  }
  const out: string[] = [];
  for (const entry of entries) {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) out.push(...(await walkYaml(full)));
    else if (entry.isFile() && (entry.name.endsWith('.yaml') || entry.name.endsWith('.yml')))
      out.push(full);
  }
  return out;
}

async function loadTagOverlay(
  ds: LoadedDataset,
  file: string,
  parts: string[],
  rel: (f: string) => string
): Promise<void> {
  const category = parts[2]?.replace(/\.ya?ml$/, '');
  if (!category || !(TAG_CATEGORIES as readonly string[]).includes(category)) {
    ds.issues.push({
      file: rel(file),
      message: `tag overlay file must live at data/i18n/<locale>/tags/<category>.yaml; got category=${category}`
    });
    return;
  }
  const raw = await readYamlFile(file, ds.issues, rel);
  if (raw === undefined) return;
  const schema = TagOverlayFiles[category as TagCategoryName];
  v.safeParse(schema, raw);
}

async function loadTagRegistries(
  ds: LoadedDataset,
  dataDir: string,
  rel: (f: string) => string
): Promise<void> {
  const tagsDir = join(dataDir, 'tags');
  for (const category of TAG_CATEGORIES) {
    const file = join(tagsDir, `${category}.yaml`);
    const raw = await readYamlFile(file, ds.issues, rel, { optional: true });
    if (raw === undefined) continue;
    const schema = TagRegistryFiles[category];
    const result = v.safeParse(schema, raw);
    if (!result.success) {
      pushIssues(rel(file), result.issues, ds.issues)
      continue;
    }
    for (const entry of result.output) {
      ds.tagRegistry.set(entry.id, { defaultLang: entry.default_lang, file: rel(file) });
    }
  }
}

function pushIssues(
  relFile: string,
  issues: v.BaseIssue<unknown>[],
  out: LoadIssue[]
) {
  for (const issue of issues) {
    out.push({ file: relFile, message: issue.message, path: formatIssuePath(issue) });
  }
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

async function readYamlFile(
  file: string,
  issues: LoadIssue[],
  rel: (f: string) => string,
  opts: { optional?: boolean } = {}
): Promise<unknown> {
  try {
    const text = await readFile(file, 'utf8');
    return parseYaml(text);
  } catch (err) {
    if (opts.optional && (err as NodeJS.ErrnoException).code === 'ENOENT') return undefined;
    issues.push({ file: rel(file), message: `failed to parse YAML: ${(err as Error).message}` });
    return undefined;
  }
}
