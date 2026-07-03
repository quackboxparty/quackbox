const KEY = 'room';

export interface RoomSession {
	room: string;
	token: string;
}

export function readSession(): RoomSession | null {
	const raw = localStorage.getItem(KEY);
	if (!raw) return null;
	try {
		const v = JSON.parse(raw) as Partial<RoomSession>;
		if (typeof v?.room === 'string' && typeof v?.token === 'string') return { room: v.room, token: v.token };
	} catch {
		// corrupt entry — treat as absent
	}
	return null;
}

export function saveSession(s: RoomSession): void {
	localStorage.setItem(KEY, JSON.stringify(s));
}

export function clearSession(): void {
	localStorage.removeItem(KEY);
}
