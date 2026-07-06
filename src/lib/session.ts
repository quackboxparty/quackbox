const STORAGE_KEY = 'room';

export interface RoomSession {
	room: string;
	token: string;
	player?: string;
}

export function readSession(): RoomSession | null {
	const raw = localStorage.getItem(STORAGE_KEY);
	if (!raw) return null;

	try {
		const parsed = JSON.parse(raw) as Partial<RoomSession>;
		if (typeof parsed?.room !== 'string' || typeof parsed?.token !== 'string') {
			return null;
		}

		const session: RoomSession = { room: parsed.room, token: parsed.token };
		// player added later — tolerate older entries that lack it
		if (typeof parsed.player === 'string') session.player = parsed.player;
		return session;
	} catch {
		// corrupt entry — treat as absent
		return null;
	}
}

export function saveSession(session: RoomSession): void {
	localStorage.setItem(STORAGE_KEY, JSON.stringify(session));
}

export function clearSession(): void {
	localStorage.removeItem(STORAGE_KEY);
}
