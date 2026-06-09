import { loadDataset, runCrossFileChecks } from './load.ts';
import type { LoadedDataset, LoadOptions } from './load.ts';
import { childLogger } from '$lib/logger';

const log = childLogger('dataset');

let dataset = $state<LoadedDataset | null>(null);
let loading = $state(false);

export function getDataset(): LoadedDataset {
	if (!dataset) throw new Error('dataset not initialized — call initDataset() first');
	return dataset;
}

export async function initDataset(opts?: LoadOptions): Promise<void> {
	loading = true;
	try {
		const ds = await loadDataset(opts);
		await runCrossFileChecks(ds);

		if (ds.issues.length > 0) {
			log.warn({ issue_count: ds.issues.length }, 'dataset loaded with issues');
			for (const i of ds.issues) {
				log.warn({ file: i.file, path: i.path }, i.message);
			}
		} else {
			log.info({ questions: ds.questions.size, packs: ds.packs.size, tags: ds.tags.size }, 'dataset ok');
		}

		dataset = ds;
	} finally {
		loading = false;
	}
}

export async function reloadDataset(): Promise<void> {
	await initDataset(dataset ? { dataDir: dataset.dataDir } : undefined);
}

export function isDatasetLoading(): boolean {
	return loading;
}
