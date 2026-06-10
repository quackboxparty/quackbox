import { Schema } from 'effect';

/**
 * Build a strict sync decoder for `schema` that rejects unknown object keys.
 * Mirrors valibot's `v.strictObject` semantics, which the data-model spec
 * relies on for translatable-field rejection in overlays.
 */
export const strictDecoder = <S extends Schema.Decoder<unknown>>(schema: S) =>
	Schema.decodeUnknownSync(schema, { onExcessProperty: 'error' });

/**
 * Build a strict Effect-ful decoder. Returns a function
 * `(input) => Effect<A, SchemaError, R>`.
 */
export const strictEffectDecoder = <S extends Schema.Decoder<unknown>>(schema: S) =>
	Schema.decodeUnknownEffect(schema, { onExcessProperty: 'error' });

/**
 * Curried strict sync decode: `decodeStrict(schema)(input)`.
 * Throws on failure (parse error or excess property).
 */
export const decodeStrict =
	<S extends Schema.Decoder<unknown>>(schema: S) =>
	(input: unknown): S['Type'] =>
		strictDecoder(schema)(input);
