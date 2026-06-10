import * as Effect from 'effect/Effect';
import * as Layer from 'effect/Layer';
import * as NodeFileSystem from '@effect/platform-node/NodeFileSystem';
import * as NodePath from '@effect/platform-node/NodePath';

import { loadDataset } from '../src/lib/server/data/load.ts';
import { runCrossFileChecks } from '../src/lib/server/data/validate.ts';

const program = Effect.gen(function* () {
	const ds = yield* loadDataset();
	const cross = yield* runCrossFileChecks(ds);
	ds.issues.push(...cross);
	return ds;
});

const runtimeLayer = Layer.mergeAll(NodeFileSystem.layer, NodePath.layer);

const ds = await Effect.runPromise(program.pipe(Effect.provide(runtimeLayer)));
if (ds.issues.length === 0) {
	const counts = `${ds.questions.size} questions, ${ds.packs.size} packs, ${ds.tags.size} tags`;
	console.log(`✓ data ok (${counts})`);
	process.exit(0);
}

const byFile = new Map<string, typeof ds.issues>();
for (const issue of ds.issues) {
	const list = byFile.get(issue.file) ?? [];
	list.push(issue);
	byFile.set(issue.file, list);
}

for (const [file, list] of byFile) {
	console.error(`\n✗ ${file}`);
	for (const i of list) {
		const where = i.path ? `  ${i.path}: ` : '  ';
		console.error(`${where}${i.message}`);
	}
}
console.error(`\n${ds.issues.length} issue(s) across ${byFile.size} file(s)`);
process.exit(1);
