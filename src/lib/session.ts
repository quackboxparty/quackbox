const STORAGE_KEY = 'room';

export interface RoomSession {
	room: string;
	token?: string;
	player?: string;
}

export function readSession(): RoomSession | null {
	const raw = localStorage.getItem(STORAGE_KEY);
	if (!raw) return null;

	try {
		const parsed = JSON.parse(raw) as Partial<RoomSession>;
		if (typeof parsed?.room !== 'string') {
			return null;
		}

		const session: RoomSession = { room: parsed.room };
		if (typeof parsed.token === 'string') session.token = parsed.token;
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
