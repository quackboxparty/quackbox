import type { PackFilter } from '../schemas/pack.ts';
import type { Question, QuestionKind, VariantName } from '../schemas/question.ts';
import type { LoadedDataset } from './load.ts';

/**
 * Precomputed index over the question dataset for fast filter queries.
 * Built once, reused for pack filter evaluation, board resolution, etc.
 */
export interface QuestionIndex {
	/** question id → kind */
	kind: Map<string, QuestionKind>;
	/** question id → set of tags */
	tags: Map<string, Set<string>>;
	/** question id → set of variant names defined on that question */
	variants: Map<string, Set<VariantName>>;
}

export function buildIndex(ds: LoadedDataset): QuestionIndex {
	const kind = new Map<string, QuestionKind>();
	const variantSet = new Map<string, Set<VariantName>>();
	const tags = new Map<string, Set<string>>();

	for (const { item } of ds.questions) {
		kind.set(item.id, item.kind);

		const vn = new Set<VariantName>();
		const variantObj = getContentVariants(item);
		for (const key of Object.keys(variantObj)) {
			if (variantObj[key] !== undefined) {
				vn.add(key as VariantName);
			}
		}
		variantSet.set(item.id, vn);

		tags.set(item.id, new Set(item.tags));
	}

	return { kind, tags, variants: variantSet };
}

/**
 * Query the question pool using a PackFilter.
 * All filter operators are ANDed together.
 * Returns question IDs matching all criteria.
 */
export function queryPool(_ds: LoadedDataset, filter: PackFilter, idx: QuestionIndex): string[] {
	// Start with all question ids
	let ids = [...idx.kind.keys()];
	const { kinds, tags_all, tags_any, tags_none, variants_any } = filter;

	if (tags_all && tags_all.length > 0) {
		ids = ids.filter((qid) => tags_all.every((tag) => idx.tags.get(qid)?.has(tag)));
	}
	if (tags_any && tags_any.length > 0) {
		ids = ids.filter((qid) => tags_any.some((tag) => idx.tags.get(qid)?.has(tag)));
	}
	if (tags_none && tags_none.length > 0) {
		ids = ids.filter((qid) => tags_none.every((tag) => !idx.tags.get(qid)?.has(tag)));
	}
	if (kinds && kinds.length > 0) {
		ids = ids.filter((qid) => {
			const k = idx.kind.get(qid);
			return k !== undefined && kinds.includes(k);
		});
	}
	if (variants_any && variants_any.length > 0) {
		ids = ids.filter((qid) => variants_any.some((vname) => idx.variants.get(qid)?.has(vname)));
	}
	if (filter.limit) {
		ids = ids.slice(0, filter.limit);
	}

	return ids;
}

/** Extract the variants object from a question, handling all three kinds. */
function getContentVariants(q: Question): Record<string, unknown> {
	if (q.kind === 'order') return {};
	return q.content.variants;
}
