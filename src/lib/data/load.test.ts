import { loadDataset, runCrossFileChecks } from '$lib/data/load';
import { mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';

async function validate(files: Record<string, string>) {
	const root = await fixture(files);
	try {
		const ds = await loadDataset({ dataDir: root });
		await runCrossFileChecks(ds);
		return ds;
	} finally {
		await destroy(root);
	}
}

async function fixture(files: Record<string, string>): Promise<string> {
	const root = await mkdtemp(join(tmpdir(), 'qbx-test-'));
	await mkdir(join(root, 'questions'), { recursive: true });
	await mkdir(join(root, 'packs'), { recursive: true });
	await mkdir(join(root, 'tags'), { recursive: true });
	await mkdir(join(root, 'i18n'), { recursive: true });
	await mkdir(join(root, 'media'), { recursive: true });
	for (const [path, content] of Object.entries(files)) {
		const full = join(root, path);
		await mkdir(join(full, '..'), { recursive: true });
		await writeFile(full, content, 'utf8');
	}
	return root;
}

async function destroy(root: string) {
	await rm(root, { force: true, recursive: true });
}

const VALID_QUESTION = `
- id: q_alpha_one
  kind: text
  tags: [subject:geo, difficulty:general]
  content:
    default_lang: en
    prompt: { text: "What is one plus one?" }
    answer: Two
    variants:
      multiple_choice:
        choices:
          - { id: two, text: Two, correct: true }
          - { id: three, text: Three }
      open:
        accepted: ["Two", "2"]
`;

const VALID_REGISTRIES = {
	'tags/audience.yaml': `[]\n`,
	'tags/difficulty.yaml': `- id: difficulty:general
  default_lang: en
  label: General
`,
	'tags/format.yaml': `[]\n`,
	'tags/region.yaml': `[]\n`,
	'tags/subject.yaml': `- id: subject:geo
  default_lang: en
  label: Geography
- id: subject:history
  default_lang: en
  label: History
`,
	'tags/warning.yaml': `[]\n`
};

const VALID_PACK = `id: pack_alpha
title: Alpha Pack
questions: [q_alpha_one]
`;

describe('happy path', () => {
	it('loads a valid question, registry, and pack with zero issues', async () => {
		const ds = await validate({
			...VALID_REGISTRIES,
			'packs/alpha.yaml': VALID_PACK,
			'questions/test.yaml': VALID_QUESTION
		});
		expect(ds.issues).toEqual([]);
		expect(ds.questions.length).toBe(1);
		expect(ds.packs.length).toBe(1);
		expect(ds.tagRegistry.size).toBe(3);
	});

	it('loads an empty data dir without errors', async () => {
		const ds = await validate({});
		expect(ds.issues).toEqual([]);
	});
});

describe('schema validation', () => {
	it('catches missing variants on a text question', async () => {
		const ds = await validate({
			...VALID_REGISTRIES,
			'questions/bad.yaml': `
- id: q_novar
  kind: text
  tags: [subject:geo]
  content:
    default_lang: en
    prompt: { text: "Hi?" }
    answer: Hi
    variants: {}
`
		});
		expect(ds.issues.length).toBeGreaterThan(0);
		expect(ds.issues[0]?.message.toLowerCase()).toContain('variant');
	});

	it('catches multiple_choice with no correct choice', async () => {
		const ds = await validate({
			...VALID_REGISTRIES,
			'questions/bad.yaml': `
- id: q_nocorrect
  kind: text
  tags: [subject:geo]
  content:
    default_lang: en
    prompt: { text: "Hi?" }
    answer: Hi
    variants:
      multiple_choice:
        choices:
          - { id: a, text: A }
          - { id: b, text: B }
`
		});
		expect(ds.issues.length).toBeGreaterThan(0);
	});

	it('catches non-contiguous order positions', async () => {
		const ds = await validate({
			...VALID_REGISTRIES,
			'questions/bad.yaml': `
- id: q_jumpy
  kind: order
  tags: [subject:geo]
  content:
    default_lang: en
    prompt: { text: "Order these." }
    items:
      - { id: a, text: A, position: 1 }
      - { id: c, text: C, position: 3 }
`
		});
		expect(ds.issues.length).toBeGreaterThan(0);
	});

	it('catches overlay with non-translatable fields', async () => {
		const ds = await validate({
			...VALID_REGISTRIES,
			'i18n/de/questions/ok.yaml': `
- id: q_alpha_one
  content:
    prompt: { text: "Was ist eins plus eins?" }
    variants:
      multiple_choice:
        choices:
          - { id: two, text: Zwei, correct: true }
`,
			'questions/ok.yaml': VALID_QUESTION
		});
		expect(ds.issues.length).toBeGreaterThan(0);
	});
});

describe('cross-file checks', () => {
	it('catches duplicate question ids', async () => {
		const ds = await validate({
			...VALID_REGISTRIES,
			'questions/a.yaml': VALID_QUESTION,
			'questions/b.yaml': VALID_QUESTION // same id q_alpha_one
		});
		expect(ds.issues.some((i) => i.message.includes('duplicate question id'))).toBe(true);
	});

	it('catches pack referencing unknown question', async () => {
		const ds = await validate({
			...VALID_REGISTRIES,
			'packs/ghost.yaml': `id: pack_ghost
title: Ghost
questions: [q_does_not_exist]
`
		});
		expect(ds.issues.some((i) => i.message.includes('unknown question'))).toBe(true);
	});

	it('catches pack includes unknown pack', async () => {
		const ds = await validate({
			...VALID_REGISTRIES,
			'packs/a.yaml': `id: pack_a
title: A
includes: [pack_missing]
questions: []
`
		});
		expect(ds.issues.some((i) => i.message.includes('unknown pack'))).toBe(true);
	});

	it('catches replaced_by pointing nowhere', async () => {
		const ds = await validate({
			...VALID_REGISTRIES,
			'questions/a.yaml': `
- id: q_old
  kind: text
  tags: [subject:geo]
  deprecated: { reason: "gone", replaced_by: q_new }
  content:
    default_lang: en
    prompt: { text: "Old?" }
    answer: Old
    variants: { open: { accepted: ["Old"] } }
`
		});
		expect(ds.issues.some((i) => i.message.includes('replaced_by'))).toBe(true);
	});

	it('catches question overlay referencing unknown id', async () => {
		const ds = await validate({
			...VALID_REGISTRIES,
			'i18n/de/questions/ghost.yaml': `
- id: q_not_real
  content:
    prompt: { text: "Kein Problem?" }
`
		});
		expect(ds.issues.some((i) => i.message.includes('overlay references unknown question'))).toBe(
			true
		);
	});

	it('catches unknown tag on question', async () => {
		const ds = await validate({
			...VALID_REGISTRIES,
			'questions/a.yaml': `
- id: q_taggy
  kind: text
  tags: [subject:nonexistent]
  content:
    default_lang: en
    prompt: { text: "Hi" }
    answer: Hi
    variants: { open: { accepted: ["Hi"] } }
`
		});
		expect(ds.issues.some((i) => i.message.includes('unknown tag'))).toBe(true);
	});

	it('catches pack includes cycle', async () => {
		const ds = await validate({
			...VALID_REGISTRIES,
			'packs/a.yaml': `id: pack_a
title: A
includes: [pack_b]
questions: []
`,
			'packs/b.yaml': `id: pack_b
title: B
includes: [pack_a]
questions: []
`
		});
		expect(ds.issues.some((i) => i.message.includes('cycle'))).toBe(true);
	});
});

describe('media checks', () => {
	it('catches missing media file', async () => {
		const ds = await validate({
			...VALID_REGISTRIES,
			'questions/a.yaml': `
- id: q_pic
  kind: text
  tags: [subject:geo]
  content:
    default_lang: en
    prompt:
      text: "Look"
      media:
        - { kind: image, ref: "local:img/flag.png" }
    answer: Red
    variants: { open: { accepted: ["Red"] } }
`
		});
		expect(ds.issues.some((i) => i.message.includes('media file missing'))).toBe(true);
	});

	it('catches extension/kind mismatch', async () => {
		const root = await fixture({
			...VALID_REGISTRIES,
			'media/img/song.mp3': 'audio bytes',
			'questions/a.yaml': `
- id: q_pic
  kind: text
  tags: [subject:geo]
  content:
    default_lang: en
    prompt:
      text: "Look"
      media:
        - { kind: image, ref: "local:img/song.mp3" }
    answer: Red
    variants: { open: { accepted: ["Red"] } }
`
		});
		try {
			const ds = await loadDataset({ dataDir: root });
			await runCrossFileChecks(ds);
			expect(ds.issues.some((i) => i.message.includes('kind mismatch'))).toBe(true);
		} finally {
			await destroy(root);
		}
	});

	it('accepts video file used as kind: audio', async () => {
		const root = await fixture({
			...VALID_REGISTRIES,
			'media/clip/vid.mp4': 'video bytes',
			'questions/a.yaml': `
- id: q_clip
  kind: text
  tags: [subject:geo]
  content:
    default_lang: en
    prompt:
      text: "Listen"
      media:
        - { kind: audio, ref: "local:clip/vid.mp4" }
    answer: Blue
    variants: { open: { accepted: ["Blue"] } }
`
		});
		try {
			const ds = await loadDataset({ dataDir: root });
			await runCrossFileChecks(ds);
			expect(ds.issues.some((i) => i.message.includes('media'))).toBe(false);
		} finally {
			await destroy(root);
		}
	});

	it('catches oversized image', async () => {
		const root = await fixture({
			...VALID_REGISTRIES,
			'media/img/huge.png': 'x'.repeat(101 * 1024), // 101 KB
			'questions/a.yaml': `
- id: q_big
  kind: text
  tags: [subject:geo]
  content:
    default_lang: en
    prompt:
      text: "Oversized"
      media:
        - { kind: image, ref: "local:img/huge.png" }
    answer: Big
    variants: { open: { accepted: ["Big"] } }
`
		});
		try {
			const ds = await loadDataset({ dataDir: root });
			await runCrossFileChecks(ds);
			expect(ds.issues.some((i) => i.message.includes('size cap'))).toBe(true);
		} finally {
			await destroy(root);
		}
	});
});
