import { stat } from 'node:fs/promises';
import { extname, join } from 'node:path';

import type { Question } from '../schemas/question.ts';
import type { LoadedDataset, LoadIssue } from './load.ts';

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

export async function runCrossFileChecks(ds: LoadedDataset): Promise<void> {
	const { issues } = ds;
	const seenQ = buildSeenMap(ds.questions);
	const seenP = buildSeenMap(ds.packs);

	checkDuplicates(ds, seenQ, seenP, issues);
	checkTagRefs(ds, seenQ, seenP, issues);
	checkRefs(ds, seenQ, seenP, issues);
	checkOverlayRefs(ds, seenQ, seenP, issues);
	checkPackCycles(ds, issues);

	const mediaDir = join(ds.dataDir, 'media');
	for (const { file, item } of ds.questions) {
		for (const m of collectQuestionMedia(item)) {
			await checkMediaFile(m.ref, m.kind, file, mediaDir, issues);
		}
	}
}

function checkPackCycles(ds: LoadedDataset, issues: LoadIssue[]): void {
	const packGraph = new Map<string, string[]>();
	for (const { item } of ds.packs) packGraph.set(item.id, item.includes ?? []);
	const fileById = new Map<string, string>();
	for (const { file, item } of ds.packs) fileById.set(item.id, file);
	issues.push(...detectCycles(packGraph, fileById));
}

function detectCycles(
	packGraph: Map<string, string[]>,
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

function checkTagRefs(
	ds: LoadedDataset,
	_seenQ: Map<string, string>,
	_seenP: Map<string, string>,
	issues: LoadIssue[]
): void {
	for (const { file, item } of ds.questions) {
		for (const t of item.tags) {
			if (!ds.tagRegistry.has(t)) {
				issues.push({ file, message: `unknown tag ${t} on question ${item.id}` });
			}
		}
	}
	for (const { file, item } of ds.packs) {
		const f = item.filter;
		const allTags = [...(f?.tags_all ?? []), ...(f?.tags_any ?? []), ...(f?.tags_none ?? [])];
		for (const t of allTags) {
			if (!ds.tagRegistry.has(t)) {
				issues.push({ file, message: `unknown tag ${t} on pack ${item.id}` });
			}
		}
	}
}

function checkRefs(
	ds: LoadedDataset,
	seenQ: Map<string, string>,
	seenP: Map<string, string>,
	issues: LoadIssue[]
): void {
	for (const { file, item } of ds.packs) {
		for (const qid of item.questions ?? []) {
			if (!seenQ.has(qid)) {
				issues.push({ file, message: `pack ${item.id} references unknown question ${qid}` });
			}
		}
		for (const pid of item.includes ?? []) {
			if (!seenP.has(pid)) {
				issues.push({ file, message: `pack ${item.id} includes unknown pack ${pid}` });
			}
		}
	}
	for (const { file, item } of ds.questions) {
		const dep = item.deprecated;
		if (dep?.replaced_by && !seenQ.has(dep.replaced_by)) {
			issues.push({
				file,
				message: `question ${item.id} replaced_by unknown question ${dep.replaced_by}`
			});
		}
	}
}

function checkOverlayRefs(
	ds: LoadedDataset,
	seenQ: Map<string, string>,
	seenP: Map<string, string>,
	issues: LoadIssue[]
): void {
	for (const [qid, list] of ds.questionOverlays) {
		if (!seenQ.has(qid)) {
			for (const o of list) {
				issues.push({ file: o.file, message: `overlay references unknown question ${qid}` });
			}
		}
	}
	for (const [pid, list] of ds.packOverlays) {
		if (!seenP.has(pid)) {
			for (const o of list) {
				issues.push({ file: o.file, message: `overlay references unknown pack ${pid}` });
			}
		}
	}
}

async function checkMediaFile(
	ref: string,
	kind: 'audio' | 'image' | 'video',
	contextFile: string,
	mediaDir: string,
	issues: LoadIssue[]
): Promise<void> {
	if (!ref.startsWith('local:')) return; // url:/youtube: are format-only
	const sub = ref.slice('local:'.length);
	const full = join(mediaDir, sub);

	let st;
	try {
		st = await stat(full);
	} catch {
		issues.push({ file: contextFile, message: `media file missing: ${ref}` });
		return;
	}

	const ext = extname(sub).toLowerCase();
	const actual = EXT_KIND_MAP[ext];
	if (!actual) {
		issues.push({ file: contextFile, message: `unknown media extension: ${ext} (${ref})` });
	} else {
		// A video file may be used as audio-only (play track, hide visuals).
		const ok = actual === kind || (actual === 'video' && kind === 'audio');
		if (!ok) {
			issues.push({
				file: contextFile,
				message: `media kind mismatch: declared ${kind} but extension ${ext} is ${actual} (${ref})`
			});
		}
	}

	const KB = 1024;
	const MB = 1024 * KB;
	const cap = kind === 'image' ? 100 * KB : 1 * MB;
	if (st.size > cap) {
		issues.push({
			file: contextFile,
			message: `media file exceeds size cap (${st.size}B > ${cap}B): ${ref}`
		});
	}
}

function checkDuplicates(
	ds: LoadedDataset,
	seenQ: Map<string, string>,
	_seenP: Map<string, string>,
	issues: LoadIssue[]
): void {
	// duplicate question ids
	for (const { file, item } of ds.questions) {
		if (seenQ.has(item.id) && seenQ.get(item.id) !== file) {
			issues.push({
				file,
				message: `duplicate question id ${item.id} (also in ${seenQ.get(item.id)})`
			});
		}
	}

	// duplicate pack ids
	const seenP = new Map<string, string>();
	for (const { file, item } of ds.packs) {
		const prev = seenP.get(item.id);
		if (prev) {
			issues.push({ file, message: `duplicate pack id ${item.id} (also in ${prev})` });
		} else {
			seenP.set(item.id, file);
		}
	}
}

function buildSeenMap(items: { file: string; item: { id: string } }[]): Map<string, string> {
	const seen = new Map<string, string>();
	for (const { file, item } of items) seen.set(item.id, file);
	return seen;
}
