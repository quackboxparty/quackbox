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

  checkTagRefs(ds, issues);
  checkRefs(ds, issues);
  checkOverlayRefs(ds, issues);
  checkPackCycles(ds, issues);

  const mediaDir = join(ds.dataDir, 'media');
  for (const { file, item } of ds.questions.values()) {
    for (const m of collectQuestionMedia(item)) {
      await checkMediaFile(m.ref, m.kind, file, mediaDir, issues);
    }
  }
}

function checkPackCycles(ds: LoadedDataset, issues: LoadIssue[]): void {
  const packGraph = new Map<string, string[]>();
  const fileById = new Map<string, string>();

  for (const { file, item } of ds.packs.values()) {
    packGraph.set(item.id, item.includes ?? []);
    fileById.set(item.id, file);
  }

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
  issues: LoadIssue[]
): void {
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

function checkRefs(
  ds: LoadedDataset,
  issues: LoadIssue[]
): void {
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

function checkOverlayRefs(
  ds: LoadedDataset,
  issues: LoadIssue[]
): void {
  for (const [locale, overlays] of ds.overlays) {
    for (const [qid, question] of overlays.questions) {
      if (!ds.questions.has(qid)) {
        issues.push({ file: question.file, message: `overlay of locale '${locale}' references unknown question '${qid}'` });
      }
    }
    for (const [pid, pack] of overlays.packs) {
      if (!ds.packs.has(pid)) {
        issues.push({ file: pack.file, message: `overlay of locale '${locale}' references unknown pack '${pid}'` });
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
