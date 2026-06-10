import { initDatasetServer } from '$lib/server/data-init';
import type { LoadedDataset, LoadOptions } from '$lib/server/data/shared';

let dataset = $state<LoadedDataset | null>(null);
let loading = $state(false);

export async function initDataset(opts?: LoadOptions): Promise<void> {
	loading = true;
	try {
		dataset = await initDatasetServer(opts);
	} finally {
		loading = false;
	}
}

export function getDataset(): LoadedDataset | null {
	return dataset;
}

export function getLoading(): boolean {
	return loading;
}
