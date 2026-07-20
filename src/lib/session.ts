const PREFIX = 'session:';
const LAST_KEY = 'session:last';
const MAX_AGE_MS = 12 * 60 * 60 * 1000;

export interface RoomSession {
	room: string;
	token?: string;
	player?: string;
}

type Stored = RoomSession & { timestamp: number };

function parse(raw: string | null): Stored | null {
	if (!raw) return null;

	try {
		const parsed = JSON.parse(raw) as Partial<Stored>;
		if (typeof parsed?.room !== 'string' || typeof parsed?.timestamp !== 'number') {
			return null;
		}

		const session: Stored = { room: parsed.room, timestamp: parsed.timestamp };
		if (typeof parsed.token === 'string') session.token = parsed.token;
		if (typeof parsed.player === 'string') session.player = parsed.player;
		return session;
	} catch {
		// corrupt entry — treat as absent
		return null;
	}
}

export function readSession(room: string): RoomSession | null {
	return parse(localStorage.getItem(PREFIX + room));
}

export function saveSession(session: RoomSession): void {
	localStorage.setItem(PREFIX + session.room, JSON.stringify({ ...session, timestamp: Date.now() }));
	localStorage.setItem(LAST_KEY, session.room);
	prune();
}

export function clearSession(room: string): void {
	localStorage.removeItem(PREFIX + room);
}

/** Most recently saved session, for the home-page rejoin button. */
export function lastSession(): RoomSession | null {
	const room = localStorage.getItem(LAST_KEY);
	return room ? readSession(room) : null;
}

function prune(): void {
	// legacy single-key session from before per-room keys
	localStorage.removeItem('room');
	for (const key of Object.keys(localStorage)) {
		if (!key.startsWith(PREFIX) || key === LAST_KEY) continue;
		const entry = parse(localStorage.getItem(key));
		if (!entry || Date.now() - entry.timestamp > MAX_AGE_MS) localStorage.removeItem(key);
	}
}
