import { type Issue } from 'effect/SchemaIssue';

/**
 * One validation problem with enough context to point the user at the
 * offending file and (structurally) where in the file it happened.
 */
export interface LoadIssue {
	file: string;
	message: string;
	path?: string;
}

/**
 * Walk an Effect `Issue` tree and flatten it into a list of `LoadIssue`.
 * Each `Pointer` node contributes one entry; its inner issue is what
 * supplies the human-readable message.
 */
export function flattenIssue(file: string, issue: Issue): LoadIssue[] {
	switch (issue._tag) {
		case 'Pointer': {
			const where = formatPath(issue.path);
			return flattenIssue(file, issue.issue).map((entry) => ({
				...entry,
				path: entry.path ? `${where}.${entry.path}` : where
			}));
		}
		case 'Composite':
		case 'AnyOf':
			return issue.issues.flatMap((inner) => flattenIssue(file, inner));
		default:
			return [{ file, message: String(issue) }];
	}
}

function formatPath(path: readonly PropertyKey[]): string {
	const parts: string[] = [];
	for (const key of path) {
		if (typeof key === 'number') parts.push(`[${key}]`);
		else if (typeof key === 'string') parts.push(parts.length === 0 ? key : `.${key}`);
	}
	return parts.join('');
}
