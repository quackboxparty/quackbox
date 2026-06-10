import { err, ok, type Result, type ResultAsync } from 'neverthrow';

import { loadDataset, runCrossFileChecks } from './load.ts';
import type { LoadedDataset, LoadIssue, LoadOptions } from './load.ts';
import { childLogger } from '$lib/logger';

const log = childLogger('dataset');

let dataset = $state<LoadedDataset | null>(null);
let loading = $state(false);

export function getDataset(): Result<LoadedDataset, 'not_initialized'> {
	if (!dataset) return err('not_initialized');
	return ok(dataset);
}

export function initDataset(opts?: LoadOptions): ResultAsync<LoadedDataset, LoadIssue[]> {
	loading = true;

	return loadDataset(opts)
		.map(async (ds) => {
			ds.issues.push(...(await runCrossFileChecks(ds)));

			if (ds.issues.length > 0) {
				log.warn({ issue_count: ds.issues.length }, 'dataset loaded with issues');
				for (const i of ds.issues) {
					log.warn({ file: i.file, path: i.path }, i.message);
				}
			} else {
				log.info(
					{ questions: ds.questions.size, packs: ds.packs.size, tags: ds.tags.size },
					'dataset ok'
				);
			}

			dataset = ds;
			return ds;
		})
		.andTee(() => {
			loading = false;
		})
		.orTee(() => {
			loading = false;
		});
}

export function reloadDataset(): ResultAsync<LoadedDataset, LoadIssue[]> {
	return initDataset(dataset ? { dataDir: dataset.dataDir } : undefined);
}

export function isDatasetLoading(): boolean {
	return loading;
}
