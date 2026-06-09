import { readdir, stat } from 'node:fs/promises';
import { dirname, join, relative } from 'node:path';

import {
  type Pack,
  PackFile,
  type PackOverlay,
  type Question,
  QuestionFile,
  type QuestionOverlay,
  QuestionOverlayFile,
  TagOverlayFiles,
  TagRegistryFiles,
  type TagOverlay,
  PackOverlayFile,
  type TagCategory,
  type Tag
} from '../schemas/index.ts';
import { DATA_DIR, TAG_CATEGORIES, type TagCategoryName } from './paths.ts';
import { parse } from './util.ts';
import type { Dirent } from 'node:fs';

export { runCrossFileChecks } from './validate.ts';

export interface LoadedDataset {
  dataDir: string;
  // TODO: maybe have different dimensions to filter faster, like tags etc.
  questions: Registry<Question>;
  packs: Registry<Pack>;
  tags: Registry<Tag>;
  overlays: Overlays
  issues: LoadIssue[];
}

export interface LoadIssue {
  file: string;
  message: string;
  path?: string;
}

export interface LoadOptions {
  dataDir?: string;
}

export type Overlays = Map<string, {
  questions: Registry<QuestionOverlay>;
  packs: Registry<PackOverlay>;
  tags: Registry<TagOverlay>;
}>;

export type Registry<T> = Map<string, Entry<T>>;

export interface Entry<T> {
  file: string;
  item: T
}

interface LoadResult<T> {
  items: T;
  issues: LoadIssue[]
}

export async function loadDataset(opts: LoadOptions = {}): Promise<LoadedDataset> {
  const dataDir = opts.dataDir ?? DATA_DIR;
  const base = dirname(dataDir);
  const rel = (file: string) => relative(base, file);

  const [questions, packs, tags, overlays] = await Promise.all([
    loadQuestions(dataDir, rel),
    loadPacks(dataDir, rel),
    loadTags(dataDir, rel),
    loadOverlays(dataDir, rel),
  ]);

  const ds: LoadedDataset = {
    dataDir,
    issues: [...questions.issues, ...packs.issues, ...tags.issues, ...overlays.issues],
    packs: packs.items,
    questions: questions.items,
    tags: tags.items,
    overlays: overlays.items
  };

  return ds;
}

async function loadQuestions(
  dataDir: string,
  rel: (f: string) => string
): Promise<LoadResult<Registry<Question>>> {
  const items = new Map<string, Entry<Question>>();
  const issues: LoadIssue[] = [];

  const questionsDir = join(dataDir, 'questions');
  const { files, issues: walkIssues } = await walkYaml(questionsDir);
  issues.push(...walkIssues);

  const results = await Promise.all(files.map((file) =>
    parse(file, QuestionFile)
      .map((questions) => ({ file, questions }))
  ));
  for (const result of results) {
    result.match(
      ({ file, questions }) => {
        for (const q of questions) {
          if (items.has(q.id)) {
            issues.push({ file, message: `duplicate question id found '${q.id}'` })
          } else {
            items.set(q.id, { file: rel(file), item: q });
          }
        };
      },
      (err) => issues.push(...err)
    )
  }

  return { items, issues };
}

async function loadPacks(
  dataDir: string,
  rel: (f: string) => string
): Promise<LoadResult<Registry<Pack>>> {
  const items = new Map<string, Entry<Pack>>();
  const issues: LoadIssue[] = [];

  const packsDir = join(dataDir, 'packs');
  const { files, issues: walkIssues } = await walkYaml(packsDir);
  issues.push(...walkIssues);

  const results = await Promise.all(files.map((file) =>
    parse(file, PackFile)
      .map((pack) => ({ file, pack }))
  ));
  for (const result of results) {
    result.match(
      ({ file, pack }) => {
        if (items.has(pack.id)) {
          issues.push({ file, message: `duplicate pack id found '${pack.id}'` })
        } else {
          items.set(pack.id, { file: rel(file), item: pack });
        }
      },
      (err) => issues.push(...err)
    )
  }

  return { items, issues };
}

async function loadTags(
  dataDir: string,
  rel: (f: string) => string
): Promise<LoadResult<Registry<Tag>>> {
  const items = new Map<string, Entry<Tag>>();
  const issues: LoadIssue[] = [];

  const tagsDir = join(dataDir, 'tags');
  for (const category of TAG_CATEGORIES) {
    const file = join(tagsDir, `${category}.yaml`);
    try { await stat(file) } catch { continue }
    await parse(file, TagRegistryFiles[category]).match(
      (tags) => {
        for (const t of tags) {
          if (items.has(t.id)) {
            issues.push({ file, message: `duplicate pack id found '${t.id}'` })
          } else {
            items.set(t.id, { file: rel(file), item: t });
          }
        }
      },
      (err) => issues.push(...err)
    )
  }

  return { items, issues };
}

async function loadOverlays(
  dataDir: string,
  rel: (f: string) => string
): Promise<LoadResult<Overlays>> {
  const overlays: Overlays = new Map()
  const issues: LoadIssue[] = [];

  const i18nDir = join(dataDir, 'i18n');
  const { files, issues: walkIssues } = await walkYaml(i18nDir);
  issues.push(...walkIssues);

  for (const file of files) {
    const r = relative(i18nDir, file);
    const parts = r.split(/[\\/]/);
    const locale = parts[0];
    const kind = parts[1];
    const filename = parts[2];
    if (!locale || !filename) continue;

    const localeOverlays = overlays.getOrInsert(locale, {
      questions: new Map(),
      packs: new Map(),
      tags: new Map()
    })

    if (kind === 'questions') {
      await parse(file, QuestionOverlayFile).match(
        (questions) => {
          for (const q of questions) {
            localeOverlays.questions.set(q.id, { file: rel(file), item: q });
          }
        },
        (err) => issues.push(...err)
      )
    } else if (kind === 'packs') {
      await parse(file, PackOverlayFile).match(
        (pack) => {
          localeOverlays.packs.set(pack.id, { file: rel(file), item: pack });
        },
        (err) => issues.push(...err)
      )
    } else if (kind === 'tags') {
      const { items, issues: tagIssues } = await loadTagOverlay(file, filename, rel)
      localeOverlays.tags = new Map([...localeOverlays.tags, ...items])
      issues.push(...tagIssues)
    } else {
      issues.push({
        file: rel(file),
        message: `unknown i18n subdirectory: ${kind ?? '(root)'}`
      });
    }
  }

  return { items: overlays, issues }
}

async function walkYaml(dir: string): Promise<{ files: string[], issues: LoadIssue[] }> {
  let entries: Dirent[];

  try {
    entries = await readdir(dir, { withFileTypes: true });
  } catch (err) {
    if ((err as NodeJS.ErrnoException).code === 'ENOENT') {
      return { files: [], issues: [{ file: dir, message: `dir doesn't exist??? ${dir}` }] };
    }
    return { files: [], issues: [{ file: dir, message: `failed to read directory: ${(err as Error).message}` }] }
  }

  const files: string[] = [];
  const issues: LoadIssue[] = []
  for (const entry of entries) {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) {
      const sub = await walkYaml(full);
      files.push(...sub.files);
      issues.push(...sub.issues);
    } else if (entry.isFile() && (entry.name.endsWith('.yaml') || entry.name.endsWith('.yml'))) {
      files.push(full);
    }
  }
  return { files, issues };
}

async function loadTagOverlay(
  file: string,
  filename: string,
  rel: (f: string) => string
): Promise<LoadResult<Registry<TagOverlay>>> {
  const tagOverlays = new Map<string, Entry<TagOverlay>>();
  const issues: LoadIssue[] = [];

  const category = filename.replace(/\.ya?ml$/, '');
  if (!category || !TAG_CATEGORIES.includes(category as TagCategory)) {
    issues.push({
      file: rel(file),
      message: `tag overlay file must live at data/i18n/<locale>/tags/<category>.yaml; got category=${category}`
    });

    return { items: tagOverlays, issues };
  }
  await parse(file, TagOverlayFiles[category as TagCategoryName]).match(
    (tags) => {
      for (const t of tags) {
        tagOverlays.set(t.id, { file: rel(file), item: t });
      }
    },
    (err) => issues.push(...err)
  )

  return { items: tagOverlays, issues };
}
