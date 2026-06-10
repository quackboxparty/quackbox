import type { PackFilter } from '../schemas/pack.ts';
import type { Question, VariantName } from '../schemas/question.ts';
import type { LoadedDataset } from './load.ts';

/**
 * Query the question pool using a PackFilter.
 * All filter operators are ANDed together.
 * Returns question IDs matching all criteria.
 */
export function queryPool(ds: LoadedDataset, filter: PackFilter): string[] {
	const { kinds, tags_all, tags_any, tags_none, variants_any, limit } = filter;

	const questions = [...ds.questions.values()];

	const matched = questions.filter((entry) => {
		const q = entry.item;

		if (kinds && kinds.length > 0 && !kinds.includes(q.kind)) return false;

		if (tags_all && tags_all.length > 0) {
			if (!tags_all.every((tag) => q.tags.includes(tag))) return false;
		}

		if (tags_any && tags_any.length > 0) {
			if (!tags_any.some((tag) => q.tags.includes(tag))) return false;
		}

		if (tags_none && tags_none.length > 0) {
			if (tags_none.some((tag) => q.tags.includes(tag))) return false;
		}

		if (variants_any && variants_any.length > 0) {
			const defined = getVariantNames(q);
			if (!variants_any.some((vname) => defined.has(vname))) return false;
		}

		return true;
	});

	const ids = matched.map((entry) => entry.item.id);
	return limit ? ids.slice(0, limit) : ids;
}

function getVariantNames(q: Question): Set<VariantName> {
	if (q.kind === 'order') return new Set();
	const variants = q.content.variants as Record<string, unknown>;
	return new Set(Object.keys(variants).filter((k) => variants[k] !== undefined) as VariantName[]);
}
