import { parse } from '$lib/data/util';
import { mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import * as v from 'valibot';

const Schema = v.object({ id: v.string() });

async function tmp() {
  return mkdtemp(join(tmpdir(), 'qbx-util-'));
}

describe('parse', () => {
  it('returns err on invalid YAML syntax', async () => {
    const dir = await tmp();
    const file = join(dir, 'broken.yaml');
    await writeFile(file, ':\n  - [unclosed', 'utf8');

    const result = await parse(file, Schema);

    result.match(
      () => expect.unreachable('should have failed'),
      (issues) => {
        expect(issues.length).toBe(1);
        expect(issues[0].message).toMatch(/invalid YAML/i);
      }
    );

    await rm(dir, { recursive: true });
  });

  it('returns err on valid YAML that fails schema', async () => {
    const dir = await tmp();
    const file = join(dir, 'bad-schema.yaml');
    await writeFile(file, 'not_an_object: true', 'utf8');

    const result = await parse(file, Schema);

    result.match(
      () => expect.unreachable('should have failed'),
      (issues) => {
        expect(issues.length).toBeGreaterThan(0);
        expect(issues[0].file).toBe(file);
      }
    );

    await rm(dir, { recursive: true });
  });

  it('returns ok on valid YAML matching schema', async () => {
    const dir = await tmp();
    const file = join(dir, 'good.yaml');
    await writeFile(file, 'id: hello', 'utf8');

    const result = await parse(file, Schema);

    result.match(
      (value) => expect(value).toEqual({ id: 'hello' }),
      () => expect.unreachable('should have succeeded')
    );

    await rm(dir, { recursive: true });
  });

  it('returns err on missing file', async () => {
    const result = await parse('/nonexistent/path.yaml', Schema);

    result.match(
      () => expect.unreachable('should have failed'),
      (issues) => {
        expect(issues.length).toBe(1);
        expect(issues[0].message).toMatch(/failed to parse YAML/i);
      }
    );
  });
});
