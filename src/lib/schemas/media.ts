import { Schema } from 'effect';

export const MediaKind = Schema.Literals(['image', 'audio', 'video']);
export type MediaKind = typeof MediaKind.Type;

// `local:` — relative path under data/media/. Disallow leading `/` and `..`.
const MediaLocalRef = Schema.String.check(
	Schema.isPattern(/^local:(?!\/)(?!.*\.\.)[A-Za-z0-9._\-/]+$/)
);

// `url:https://…` — https only, scheme validated at the prefix level.
const MediaUrlRef = Schema.String.check(Schema.isPattern(/^url:https:\/\/\S+$/));

// `youtube:<id>` with optional `?start=…&end=…` style query.
// YouTube IDs are currently always 11 chars but Google has reserved the right
// to extend them — accept a generous range instead of hard-coding 11.
const MediaYoutubeRef = Schema.String.check(
	Schema.isPattern(/^youtube:[A-Za-z0-9_-]{8,24}(\?[A-Za-z0-9_=&-]*)?$/)
);

export const MediaRef = Schema.Union([MediaLocalRef, MediaUrlRef, MediaYoutubeRef]);
export type MediaRef = typeof MediaRef.Type;

export const Media = Schema.Struct({
	alt: Schema.optionalKey(Schema.String),
	// type-specific hints; loader/UI may use them, schema only sanity-checks.
	duration_ms: Schema.optionalKey(
		Schema.Number.check(Schema.isInt(), Schema.isGreaterThanOrEqualTo(0))
	),
	end_ms: Schema.optionalKey(Schema.Number.check(Schema.isInt(), Schema.isGreaterThanOrEqualTo(1))),
	height: Schema.optionalKey(Schema.Number.check(Schema.isInt(), Schema.isGreaterThanOrEqualTo(1))),
	kind: MediaKind,
	ref: MediaRef,
	// Playback segment for audio/video. Both in milliseconds, relative to
	// the source start. Omit either side to play from start / to end.
	start_ms: Schema.optionalKey(
		Schema.Number.check(Schema.isInt(), Schema.isGreaterThanOrEqualTo(0))
	),
	width: Schema.optionalKey(Schema.Number.check(Schema.isInt(), Schema.isGreaterThanOrEqualTo(1)))
}).check(
	Schema.makeFilter((m) =>
		m.start_ms === undefined || m.end_ms === undefined || m.end_ms > m.start_ms
			? undefined
			: 'media end_ms must be greater than start_ms'
	),
	Schema.makeFilter((m) =>
		(m.start_ms === undefined && m.end_ms === undefined) || m.kind !== 'image'
			? undefined
			: 'media start_ms/end_ms only apply to audio or video'
	)
);
export type Media = typeof Media.Type;

export const MediaList = Schema.Array(Media);
export type MediaList = typeof MediaList.Type;
