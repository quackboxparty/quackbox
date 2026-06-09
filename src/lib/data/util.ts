import { err, ok, ResultAsync } from 'neverthrow';
import { readFile } from 'node:fs/promises';
import * as v from 'valibot';
import { parse as parseYaml } from 'yaml';

import type { LoadIssue } from './load';

export function parse<Schema extends v.GenericSchema>(
  file: string,
  schema: Schema,
  opts: { optional?: boolean } = {}
): ResultAsync<v.InferOutput<Schema>, LoadIssue[]> {
  return readYaml(file, opts)
    .mapErr((e) => [e])
    .andThen((raw) => {
      const result = v.safeParse(schema, raw);
      return result.success
        ? ok(result.output)
        : err(mapIssues(file, result.issues));
    });
}

export function readYaml(
  file: string,
  opts: { optional?: boolean } = {}
): ResultAsync<unknown, LoadIssue> {
  return ResultAsync.fromPromise(
    readFile(file, 'utf8'),
    (error) => {
      if (opts.optional && (error as NodeJS.ErrnoException).code === 'ENOENT') {
        return { file, message: `file not found: ${(error as Error).message}` };
      }
      return { file, message: `failed to parse YAML: ${(error as Error).message}` };
    }
  ).andThen((text) => {
    // FIXME: parseYaml could error
    return ok(parseYaml(text));
  });
}

function mapIssues(
  file: string,
  issues: v.BaseIssue<unknown>[],
): LoadIssue[] {
  return issues.map((issue) => ({ file, message: issue.message, path: formatIssuePath(issue) }));
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

