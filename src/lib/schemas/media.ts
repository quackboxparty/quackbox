import * as v from 'valibot';

export const MediaKind = v.picklist(['image', 'audio', 'video']);
export type MediaKind = v.InferOutput<typeof MediaKind>;

// `local:` — relative path under data/media/. Disallow leading `/` and `..`.
const MediaLocalRef = v.pipe(v.string(), v.regex(/^local:(?!\/)(?!.*\.\.)[A-Za-z0-9._\-/]+$/));

// `url:https://…` — https only, scheme validated at the prefix level.
const MediaUrlRef = v.pipe(v.string(), v.regex(/^url:https:\/\/\S+$/));

// `youtube:<id>` with optional `?start=…&end=…` style query.
// YouTube IDs are currently always 11 chars but Google has reserved the right
// to extend them — accept a generous range instead of hard-coding 11.
const MediaYoutubeRef = v.pipe(
	v.string(),
	v.regex(/^youtube:[A-Za-z0-9_-]{8,24}(\?[A-Za-z0-9_=&-]*)?$/)
);

export const MediaRef = v.union([MediaLocalRef, MediaUrlRef, MediaYoutubeRef]);
export type MediaRef = v.InferOutput<typeof MediaRef>;

export const Media = v.pipe(
	v.strictObject({
		alt: v.optional(v.string()),
		// type-specific hints; loader/UI may use them, schema only sanity-checks.
		duration_ms: v.optional(v.pipe(v.number(), v.integer(), v.minValue(0))),
		end_ms: v.optional(v.pipe(v.number(), v.integer(), v.minValue(1))),
		height: v.optional(v.pipe(v.number(), v.integer(), v.minValue(1))),
		kind: MediaKind,
		ref: MediaRef,
		// Playback segment for audio/video. Both in milliseconds, relative to
		// the source start. Omit either side to play from start / to end.
		start_ms: v.optional(v.pipe(v.number(), v.integer(), v.minValue(0))),
		width: v.optional(v.pipe(v.number(), v.integer(), v.minValue(1)))
	}),
	v.check(
		(m) => m.start_ms === undefined || m.end_ms === undefined || m.end_ms > m.start_ms,
		'media end_ms must be greater than start_ms'
	),
	v.check(
		(m) => (m.start_ms === undefined && m.end_ms === undefined) || m.kind !== 'image',
		'media start_ms/end_ms only apply to audio or video'
	)
);
export type Media = v.InferOutput<typeof Media>;

export const MediaList = v.array(Media);
