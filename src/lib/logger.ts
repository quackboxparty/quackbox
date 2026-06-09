import pino from 'pino';

const transport = import.meta.env.DEV
	? pino.transport({ target: 'pino-pretty', options: { destination: 1 } })
	: undefined;

export const log = pino(
	{
		name: 'quackbox',
		level: import.meta.env.DEV ? 'debug' : 'info'
		// Inject OTEL trace_id/span_id when @opentelemetry/api is wired up:
		// mixins: () => {
		//   const span = trace.getSpan(context.active());
		//   if (!span) return {};
		//   const c = span.spanContext();
		//   return { trace_id: c.traceId, span_id: c.spanId };
		// }
	},
	transport
);

// Add OTLP transport later:
// const transport = pino.transport({
//   targets: [
//     { target: 'pino-pretty', options: { destination: 1 }, level: 'debug' },
//     { target: 'pino-opentelemetry-transport', options: { destination: 'http://localhost:4318/v1/logs' }, level: 'info' }
//   ]
// });

/** Create a child logger scoped to a subsystem (e.g. `childLogger('dataset')`). */
export function childLogger(name: string): pino.Logger {
	return log.child({ component: name });
}
