import type {
	Pack,
	PackOverlay,
	Question,
	QuestionOverlay,
	Tag,
	TagOverlay
} from '../../schemas/index.ts';

/**
 * One validation problem with enough context to point the user at the
 * offending file and (structurally) where in the file it happened.
 */
export interface LoadIssue {
	file: string;
	message: string;
	path?: string;
}

export interface LoadOptions {
	dataDir?: string;
}

export type Registry<T> = Map<string, Entry<T>>;

export interface Entry<T> {
	file: string;
	item: T;
}

export type Overlays = Map<
	string,
	{
		questions: Registry<QuestionOverlay>;
		packs: Registry<PackOverlay>;
		tags: Registry<TagOverlay>;
	}
>;

/** The full set of YAML-derived data, ready for cross-file checks and querying. */
export interface LoadedDataset {
	dataDir: string;
	issues: LoadIssue[];
	questions: Registry<Question>;
	packs: Registry<Pack>;
	tags: Registry<Tag>;
	overlays: Overlays;
}
