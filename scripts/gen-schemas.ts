import * as Effect from 'effect/Effect';
import * as Layer from 'effect/Layer';
import * as FileSystem from 'effect/FileSystem';
import * as Path from 'effect/Path';
import { type Top, toJsonSchemaDocument } from 'effect/Schema';
import * as NodeFileSystem from '@effect/platform-node/NodeFileSystem';
import * as NodePath from '@effect/platform-node/NodePath';

import {
	BoardFile,
	PackFile,
	PackOverlay,
	QuestionFile,
	QuestionOverlayFile,
	TagOverlayFiles,
	TagRegistryFiles
} from '../src/lib/schemas/index.ts';

const program = Effect.gen(function* () {
	const fs = yield* FileSystem.FileSystem;
	const path = yield* Path.Path;

	const outDir = path.join(import.meta.dirname, '..', 'schemas');
	yield* fs.makeDirectory(outDir, { recursive: true });

	const targets: [string, Top][] = [
		['question.schema.json', QuestionFile],
		['question-overlay.schema.json', QuestionOverlayFile],
		['pack.schema.json', PackFile],
		['pack-overlay.schema.json', PackOverlay],
		['board.schema.json', BoardFile]
	];

	for (const category of Object.keys(TagRegistryFiles) as (keyof typeof TagRegistryFiles)[]) {
		targets.push([`tag-registry-${category}.schema.json`, TagRegistryFiles[category]]);
		targets.push([`tag-overlay-${category}.schema.json`, TagOverlayFiles[category]]);
	}

	for (const [filename, schema] of targets) {
		const doc = toJsonSchemaDocument(schema);
		const body = JSON.stringify(doc, null, 2) + '\n';
		yield* fs.writeFileString(path.join(outDir, filename), body);
		console.log(`wrote schemas/${filename}`);
	}
});

await Effect.runPromise(
	program.pipe(Effect.provide(Layer.mergeAll(NodeFileSystem.layer, NodePath.layer)))
);
