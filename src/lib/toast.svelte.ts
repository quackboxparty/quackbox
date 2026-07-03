export type ToastVariant = 'success' | 'error' | 'info';

export type Toast = {
	id: number;
	variant: ToastVariant;
	message: string;
};

const MAX_VISIBLE = 3;

let nextId = 0;

export const toasts = $state<Toast[]>([]);

function push(variant: ToastVariant, message: string): number {
	const id = ++nextId;
	toasts.push({ id, variant, message });
	if (toasts.length > MAX_VISIBLE) toasts.shift();
	return id;
}

export function dismiss(id: number) {
	const i = toasts.findIndex((t) => t.id === id);
	if (i !== -1) toasts.splice(i, 1);
}

export const toast = {
	success: (message: string) => push('success', message),
	error: (message: string) => push('error', message),
	info: (message: string) => push('info', message)
};
