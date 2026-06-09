import { loadDataset, runCrossFileChecks } from '../src/lib/data/load.ts';

const ds = await loadDataset();
await runCrossFileChecks(ds);

if (ds.issues.length === 0) {
  const counts = `${ds.questions.size} questions, ${ds.packs.size} packs, ${ds.tags.size} tags`;
  console.log(`✓ data ok (${counts})`);
  process.exit(0);
}

// Group by file for readable output.
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
