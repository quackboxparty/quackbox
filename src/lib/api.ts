import type { CreateRoom, Room } from './bindings/Rooms';
import type { Game } from './bindings/Games.ts';

const API = '/api';

export type Result<T, E = ApiError> = { ok: true; value: T } | { ok: false; error: E };

export type ApiError = { kind: 'network' } | { kind: 'http'; status: number; body: string };

async function send(path: string, init?: RequestInit): Promise<Result<Response>> {
	let res: Response;
	try {
		res = await fetch(API + path, init);
	} catch {
		return { ok: false, error: { kind: 'network' } };
	}
	if (!res.ok) {
		return { ok: false, error: { kind: 'http', status: res.status, body: await res.text() } };
	}
	return { ok: true, value: res };
}

async function req<T>(path: string, init?: RequestInit): Promise<Result<T>> {
	const res = await send(path, init);
	if (!res.ok) return res;
	return { ok: true, value: (await res.value.json()) as T };
}

async function post<R, T>(path: string, body: R, init?: RequestInit): Promise<Result<T>> {
	return await req<T>(path, {
		method: 'POST',
		headers: { 'content-type': 'application/json' },
		body: JSON.stringify(body),
		...init
	});
}

async function roomExists(code: string): Promise<Result<boolean>> {
	const res = await send(`/rooms/${code}`);
	if (res.ok) return { ok: true, value: true };
	if (res.error.kind === 'http' && res.error.status === 404) {
		return { ok: true, value: false };
	}
	return res;
}

export const api = {
	room: {
		exists: roomExists,
		create: (secret: string, gameId: string) =>
			post<CreateRoom, Room>('/rooms', { secret, game_id: gameId })
	},
	games: {
		list: () => req<Game[]>('/games')
	}
};
