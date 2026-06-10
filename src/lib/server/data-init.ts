import * as Effect from 'effect/Effect';
import * as Layer from 'effect/Layer';
import { make as makeRuntime } from 'effect/ManagedRuntime';
import * as NodeFileSystem from '@effect/platform-node/NodeFileSystem';
import * as NodePath from '@effect/platform-node/NodePath';

import { loadDataset } from './data/load.ts';
import { runCrossFileChecks } from './data/validate.ts';
import type { LoadedDataset, LoadOptions } from './data/shared.ts';

/**
 * Server-only Effect runtime for the data layer. Holds the in-memory
 * `ManagedRuntime` so the FileSystem and Path services are constructed
 * once at module load, not per request. The runtime is exposed as a
 * plain-Promise wrapper so the SvelteKit boundary (`store.svelte.ts`,
 * `hooks.server.ts`) never imports the Effect runtime value.
 */
const DataLive = Layer.mergeAll(NodeFileSystem.layer, NodePath.layer);

const runtime = makeRuntime(DataLive);

export async function initDatasetServer(opts?: LoadOptions): Promise<LoadedDataset> {
	const program = Effect.gen(function* () {
		const ds = yield* loadDataset(opts);
		const cross = yield* runCrossFileChecks(ds);
		ds.issues.push(...cross);
		return ds;
	});
	return runtime.runPromise(program);
}
