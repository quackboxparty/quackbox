import { decodeStrict } from '$lib/schemas/decode';
import { Media, MediaRef } from '$lib/schemas/media';
import { describe, expect, it } from 'vitest';

const decode = decodeStrict;

describe('media', () => {
	it('accepts valid media refs', () => {
		expect(decode(MediaRef)('local:img/flag.svg')).toBe('local:img/flag.svg');
		expect(decode(MediaRef)('url:https://example.com/clip.mp3')).toBe(
			'url:https://example.com/clip.mp3'
		);
		expect(decode(MediaRef)('youtube:abcDEF12345')).toBe('youtube:abcDEF12345');
		expect(decode(MediaRef)('youtube:abcDEF12345?start=10&end=18')).toBe(
			'youtube:abcDEF12345?start=10&end=18'
		);
	});

	it('rejects invalid media refs', () => {
		expect(() => decode(MediaRef)('local:/abs/path.png')).toThrow(); // leading /
		expect(() => decode(MediaRef)('local:../../../etc/passwd')).toThrow(); // path traversal
		expect(() => decode(MediaRef)('url:http://insecure.com')).toThrow(); // http
		expect(() => decode(MediaRef)('ftp://x.com')).toThrow();
	});

	it('accepts valid media objects', () => {
		expect(decode(Media)({ kind: 'image', ref: 'local:img/a.png' })).toEqual({
			kind: 'image',
			ref: 'local:img/a.png'
		});
		expect(
			decode(Media)({
				end_ms: 10000,
				kind: 'audio',
				ref: 'local:audio/clip.ogg',
				start_ms: 2000
			})
		).toEqual({ end_ms: 10000, kind: 'audio', ref: 'local:audio/clip.ogg', start_ms: 2000 });
	});

	it('rejects media with end_ms < start_ms', () => {
		expect(() =>
			decode(Media)({
				end_ms: 1000,
				kind: 'audio',
				ref: 'url:https://example.com/x.mp3',
				start_ms: 10000
			})
		).toThrow();
	});

	it('rejects start_ms/end_ms on image kind', () => {
		expect(() =>
			decode(Media)({
				kind: 'image',
				ref: 'local:img/a.png',
				start_ms: 0
			})
		).toThrow();
	});

	it('accepts kind: audio with youtube ref', () => {
		expect(decode(Media)({ kind: 'audio', ref: 'youtube:aB3dE5fGh12' })).toEqual({
			kind: 'audio',
			ref: 'youtube:aB3dE5fGh12'
		});
	});
});
