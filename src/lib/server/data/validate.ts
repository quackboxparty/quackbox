import * as Effect from 'effect/Effect';
import * as FileSystem from 'effect/FileSystem';
import * as Path from 'effect/Path';

import type { Question } from '../../schemas/index.ts';

import type { LoadedDataset, LoadIssue } from './shared.ts';

const KB = 1024;
const MB = 1024 * KB;
const IMAGE_CAP = 100 * KB;
const MEDIA_CAP = 1 * MB;

const EXT_KIND_MAP: Record<string, 'audio' | 'image' | 'video'> = {
	'.avif': 'image',
	'.flac': 'audio',
	'.gif': 'image',
	'.jpeg': 'image',
	'.jpg': 'image',
	'.m4a': 'audio',
	'.mov': 'video',
	'.mp3': 'audio',
	'.mp4': 'video',
	'.ogg': 'audio',
	'.opus': 'audio',
	'.png': 'image',
	'.svg': 'image',
	'.wav': 'audio',
	'.webm': 'video',
	'.webp': 'image'
};

/**
 * Cross-file validation that can't be expressed inside a single schema —
 * tag/pack/question refs and media file presence. Returns the issues
 * without failing the effect (we accumulate by design).
 */
export const runCrossFileChecks = (
	ds: LoadedDataset
): Effect.Effect<LoadIssue[], never, FileSystem.FileSystem | Path.Path> =>
	Effect.gen(function* () {
		const fs = yield* FileSystem.FileSystem;
		const path = yield* Path.Path;
		const issues: LoadIssue[] = [];

		checkTagRefs(ds, issues);
		checkRefs(ds, issues);
		checkOverlayRefs(ds, issues);
		checkPackCycles(ds, issues);

		const mediaDir = path.join(ds.dataDir, 'media');
		for (const { file, item } of ds.questions.values()) {
			for (const m of collectQuestionMedia(item)) {
				yield* checkMediaFile(m.ref, m.kind, file, mediaDir, fs, path, issues);
			}
		}
		return issues;
	});

function checkPackCycles(ds: LoadedDataset, issues: LoadIssue[]): void {
	const packGraph = new Map<string, readonly string[]>();
	const fileById = new Map<string, string>();

	for (const { file, item } of ds.packs.values()) {
		packGraph.set(item.id, item.includes ?? []);
		fileById.set(item.id, file);
	}

	issues.push(...detectCycles(packGraph, fileById));
}

function detectCycles(
	packGraph: Map<string, readonly string[]>,
	fileById: Map<string, string>
): LoadIssue[] {
	const issues: LoadIssue[] = [];
	const visited = new Set<string>();
	const inProgress = new Set<string>();

	const dfs = (node: string, stack: string[]): void => {
		if (visited.has(node)) return;
		if (inProgress.has(node)) {
			const cycle = [...stack.slice(stack.indexOf(node)), node].join(' -> ');
			issues.push({
				file: fileById.get(node) ?? '(unknown)',
				message: `pack includes cycle: ${cycle}`
			});
			return;
		}

		inProgress.add(node);
		for (const next of packGraph.get(node) ?? []) {
			if (packGraph.has(next)) dfs(next, [...stack, node]);
		}
		inProgress.delete(node);
		visited.add(node);
	};

	for (const id of packGraph.keys()) dfs(id, []);
	return issues;
}

function collectQuestionMedia(q: Question): { kind: 'audio' | 'image' | 'video'; ref: string }[] {
	const out: { kind: 'audio' | 'image' | 'video'; ref: string }[] = [];
	for (const m of q.content.prompt.media ?? []) out.push({ kind: m.kind, ref: m.ref });
	if (q.kind === 'order') {
		for (const it of q.content.items)
			for (const m of it.media ?? []) out.push({ kind: m.kind, ref: m.ref });
	} else {
		const mc = q.content.variants.multiple_choice;
		if (mc)
			for (const c of mc.choices)
				for (const m of c.media ?? []) out.push({ kind: m.kind, ref: m.ref });
	}
	return out;
}

function checkTagRefs(ds: LoadedDataset, issues: LoadIssue[]): void {
	for (const { file, item } of ds.questions.values()) {
		for (const t of item.tags) {
			if (!ds.tags.has(t)) {
				issues.push({ file, message: `unknown tag ${t} on question ${item.id}` });
			}
		}
	}
	for (const { file, item } of ds.packs.values()) {
		const f = item.filter;
		const allTags = [...(f?.tags_all ?? []), ...(f?.tags_any ?? []), ...(f?.tags_none ?? [])];
		for (const t of allTags) {
			if (!ds.tags.has(t)) {
				issues.push({ file, message: `unknown tag ${t} on pack ${item.id}` });
			}
		}
	}
}

function checkRefs(ds: LoadedDataset, issues: LoadIssue[]): void {
	for (const { file, item } of ds.packs.values()) {
		for (const qid of item.questions ?? []) {
			if (!ds.questions.has(qid)) {
				issues.push({ file, message: `pack ${item.id} references unknown question ${qid}` });
			}
		}
		for (const pid of item.includes ?? []) {
			if (!ds.packs.has(pid)) {
				issues.push({ file, message: `pack ${item.id} includes unknown pack ${pid}` });
			}
		}
	}
	for (const { file, item } of ds.questions.values()) {
		const dep = item.deprecated;
		if (dep?.replaced_by && !ds.questions.has(dep.replaced_by)) {
			issues.push({
				file,
				message: `question ${item.id} replaced_by unknown question ${dep.replaced_by}`
			});
		}
	}
}

function checkOverlayRefs(ds: LoadedDataset, issues: LoadIssue[]): void {
	for (const [locale, overlays] of ds.overlays) {
		for (const [qid, question] of overlays.questions) {
			if (!ds.questions.has(qid)) {
				issues.push({
					file: question.file,
					message: `overlay of locale '${locale}' references unknown question '${qid}'`
				});
			}
		}
		for (const [pid, pack] of overlays.packs) {
			if (!ds.packs.has(pid)) {
				issues.push({
					file: pack.file,
					message: `overlay of locale '${locale}' references unknown pack '${pid}'`
				});
			}
		}
	}
}

function checkMediaFile(
	ref: string,
	kind: 'audio' | 'image' | 'video',
	contextFile: string,
	mediaDir: string,
	fs: FileSystem.FileSystem,
	path: Path.Path,
	issues: LoadIssue[]
): Effect.Effect<void, never, FileSystem.FileSystem> {
	if (!ref.startsWith('local:')) return Effect.void;
	const sub = ref.slice('local:'.length);
	const full = path.join(mediaDir, sub);

	return Effect.gen(function* () {
		const stat = yield* fs.stat(full).pipe(Effect.option);
		if (stat._tag === 'None') {
			issues.push({ file: contextFile, message: `media file missing: ${ref}` });
			return;
		}
		const ext = path.extname(sub).toLowerCase();
		const actual = EXT_KIND_MAP[ext];
		if (!actual) {
			issues.push({ file: contextFile, message: `unknown media extension: ${ext} (${ref})` });
		} else {
			const ok = actual === kind || (actual === 'video' && kind === 'audio');
			if (!ok) {
				issues.push({
					file: contextFile,
					message: `media kind mismatch: declared ${kind} but extension ${ext} is ${actual} (${ref})`
				});
			}
		}

		const cap = kind === 'image' ? IMAGE_CAP : MEDIA_CAP;
		if (stat.value.size > cap) {
			issues.push({
				file: contextFile,
				message: `media file exceeds size cap (${stat.value.size}B > ${cap}B): ${ref}`
			});
		}
	});
}
