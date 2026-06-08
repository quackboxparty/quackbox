import { Media, MediaRef } from '$lib/schemas/media';
import * as v from 'valibot';
import { describe, expect, it } from 'vitest';

describe('media', () => {
	it('accepts valid media refs', () => {
		expect(v.safeParse(MediaRef, 'local:img/flag.svg').success).toBe(true);
		expect(v.safeParse(MediaRef, 'url:https://example.com/clip.mp3').success).toBe(true);
		expect(v.safeParse(MediaRef, 'youtube:abcDEF12345').success).toBe(true);
		expect(v.safeParse(MediaRef, 'youtube:abcDEF12345?start=10&end=18').success).toBe(true);
	});

	it('rejects invalid media refs', () => {
		expect(v.safeParse(MediaRef, 'local:/abs/path.png').success).toBe(false); // leading /
		expect(v.safeParse(MediaRef, 'local:../../../etc/passwd').success).toBe(false); // path traversal
		expect(v.safeParse(MediaRef, 'url:http://insecure.com').success).toBe(false); // http
		expect(v.safeParse(MediaRef, 'ftp://x.com').success).toBe(false);
	});

	it('accepts valid media objects', () => {
		expect(v.safeParse(Media, { kind: 'image', ref: 'local:img/a.png' }).success).toBe(true);
		expect(
			v.safeParse(Media, {
				end_ms: 10000,
				kind: 'audio',
				ref: 'local:audio/clip.ogg',
				start_ms: 2000
			}).success
		).toBe(true);
	});

	it('rejects media with end_ms < start_ms', () => {
		const result = v.safeParse(Media, {
			end_ms: 1000,
			kind: 'audio',
			ref: 'url:https://example.com/x.mp3',
			start_ms: 10000
		});
		expect(result.success).toBe(false);
	});

	it('rejects start_ms/end_ms on image kind', () => {
		const result = v.safeParse(Media, {
			kind: 'image',
			ref: 'local:img/a.png',
			start_ms: 0
		});
		expect(result.success).toBe(false);
	});

	it('accepts kind: audio with youtube ref', () => {
		expect(v.safeParse(Media, { kind: 'audio', ref: 'youtube:aB3dE5fGh12' }).success).toBe(true);
	});
});
