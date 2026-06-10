import * as Effect from 'effect/Effect';
import * as Layer from 'effect/Layer';
import * as NodeFileSystem from '@effect/platform-node/NodeFileSystem';
import * as NodePath from '@effect/platform-node/NodePath';
import { String, Struct, type Decoder } from 'effect/Schema';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';

import { parse } from './util.ts';

const TestSchema = Struct({ id: String });

const Live = Layer.mergeAll(NodeFileSystem.layer, NodePath.layer);

async function tmp() {
	return mkdtemp(join(tmpdir(), 'qbx-util-'));
}

const runParse = (file: string, schema: Decoder<unknown>) =>
	Effect.runPromise(
		parse(file, schema).pipe(
			Effect.provide(Live),
			Effect.match({
				onSuccess: (value) => ({ _tag: 'Success' as const, value }),
				onFailure: (failure) => ({ _tag: 'Failure' as const, failure })
			})
		)
	);

describe('parse', () => {
	it('returns err on invalid YAML syntax', async () => {
		const dir = await tmp();
		const file = join(dir, 'broken.yaml');
		await writeFile(file, ':\n  - [unclosed', 'utf8');

		const result = await runParse(file, TestSchema);
		expect(result._tag).toBe('Failure');
		if (result._tag === 'Failure') {
			const issue = (result.failure as { message?: string }[])[0];
			expect(issue?.message ?? '').toMatch(/invalid YAML/i);
		}

		await rm(dir, { recursive: true });
	});

	it('returns err on valid YAML that fails schema', async () => {
		const dir = await tmp();
		const file = join(dir, 'bad-schema.yaml');
		await writeFile(file, 'not_an_object: true', 'utf8');

		const result = await runParse(file, TestSchema);
		expect(result._tag).toBe('Failure');
		if (result._tag === 'Failure') {
			const issue = (result.failure as { file?: string }[])[0];
			expect(issue?.file).toBe(file);
		}

		await rm(dir, { recursive: true });
	});

	it('returns ok on valid YAML matching schema', async () => {
		const dir = await tmp();
		const file = join(dir, 'good.yaml');
		await writeFile(file, 'id: hello', 'utf8');

		const result = await runParse(file, TestSchema);
		expect(result._tag).toBe('Success');
		if (result._tag === 'Success') {
			expect(result.value).toEqual({ id: 'hello' });
		}

		await rm(dir, { recursive: true });
	});

	it('returns err on missing file', async () => {
		const result = await runParse('/nonexistent/path.yaml', TestSchema);
		expect(result._tag).toBe('Failure');
		if (result._tag === 'Failure') {
			const issue = (result.failure as { message?: string }[])[0];
			expect(issue?.message ?? '').toMatch(/failed to parse YAML/i);
		}
	});
});
