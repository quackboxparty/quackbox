import * as Effect from 'effect/Effect';
import * as FileSystem from 'effect/FileSystem';
import * as Schema from 'effect/Schema';
import { isIssue, type Issue } from 'effect/SchemaIssue';
import { parse as parseYaml } from 'yaml';

import { flattenIssue, type LoadIssue } from './issue.ts';

/**
 * Read a file, parse it as YAML, then validate it against `schema`. On any
 * failure (missing file, invalid YAML, schema mismatch) the effect fails
 * with one or more `LoadIssue`s, each carrying the originating file and
 * (where the schema allows it) a path into the document.
 *
 * Strict-key rejection (`onExcessProperty: "error"`) is set so unknown
 * fields fail loudly — the data-model spec relies on this for translatable
 * fields, question overlays, and similar shape-pinned files.
 */
export function parse<S extends Schema.Decoder<unknown>>(
	file: string,
	schema: S
): Effect.Effect<S['Type'], LoadIssue[], FileSystem.FileSystem> {
	return readYaml(file).pipe(
		Effect.flatMap((raw) => decodeStrict(file, schema, raw)),
		Effect.mapError((err) => (Array.isArray(err) ? err : [err]))
	);
}

const decodeStrict = <S extends Schema.Decoder<unknown>>(
	file: string,
	schema: S,
	raw: unknown
): Effect.Effect<S['Type'], LoadIssue[]> =>
	Effect.try({
		try: () => Schema.decodeUnknownSync(schema, { onExcessProperty: 'error' })(raw),
		catch: (error) => flattenIssue(file, unwrapIssue(error))
	});

/**
 * Read a UTF-8 text file and parse it as YAML. Surfaces parse errors as
 * `LoadIssue`s rather than throwing, so callers can keep accumulating.
 */
export function readYaml(file: string): Effect.Effect<unknown, LoadIssue[], FileSystem.FileSystem> {
	return Effect.gen(function* () {
		const fs = yield* FileSystem.FileSystem;
		const text = yield* fs.readFileString(file).pipe(
			Effect.mapError((error: unknown): LoadIssue[] => [
				{
					file,
					message: `failed to parse YAML: ${error instanceof Error ? error.message : String(error)}`
				}
			])
		);
		try {
			return parseYaml(text) as unknown;
		} catch (error) {
			return yield* Effect.fail([
				{
					file,
					message: `invalid YAML: ${error instanceof Error ? error.message : String(error)}`
				} satisfies LoadIssue
			]);
		}
	});
}

/** Unwrap a sync `Schema.decodeUnknownSync` throw into the inner `Issue` tree. */
function unwrapIssue(error: unknown): Issue {
	if (error !== null && typeof error === 'object' && 'cause' in error && isIssue(error.cause)) {
		return (error as { cause: Issue }).cause;
	}
	return error as Issue;
}
