import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { toJsonSchema } from '@valibot/to-json-schema';
import type { BaseSchema, BaseIssue } from 'valibot';

import {
	PackOverlay,
	PackFile,
	QuestionFile,
	QuestionOverlayFile,
	TagOverlayFiles,
	TagRegistryFiles
} from '../src/lib/schemas/index.ts';

const here = dirname(fileURLToPath(import.meta.url));
const outDir = join(here, '..', 'schemas');

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type AnySchema = BaseSchema<unknown, unknown, BaseIssue<unknown>>;

const targets: Array<[string, AnySchema]> = [
	['question.schema.json', QuestionFile],
	['question-overlay.schema.json', QuestionOverlayFile],
	['pack.schema.json', PackFile],
	['pack-overlay.schema.json', PackOverlay]
];

for (const category of Object.keys(TagRegistryFiles) as Array<keyof typeof TagRegistryFiles>) {
	targets.push([`tag-registry-${category}.schema.json`, TagRegistryFiles[category]]);
	targets.push([`tag-overlay-${category}.schema.json`, TagOverlayFiles[category]]);
}

await mkdir(outDir, { recursive: true });

for (const [filename, schema] of targets) {
	const jsonSchema = toJsonSchema(schema, {
		errorMode: 'warn'
	});
	const body = JSON.stringify(jsonSchema, null, 2) + '\n';
	await writeFile(join(outDir, filename), body, 'utf8');
	console.log(`wrote schemas/${filename}`);
}
