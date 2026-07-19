<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { api } from '$lib/api';
	import type { ClientMessage, ClientView, ServerMessage } from '$lib/bindings/Protocol';
	import Button from '$lib/components/Button.svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import GameStage from '$lib/components/game/GameStage.svelte';
	import TextInput from '$lib/components/TextInput.svelte';
	import { m } from '$lib/paraglide/messages';
	import { readSession, saveSession } from '$lib/session';
	import { room, clearRoom } from '$lib/room.svelte';
	import { toast } from '$lib/toast.svelte';
	import { onMount } from 'svelte';

	let nameOpen = $state(false);
	let name = $state('');
	let snapshot = $state<ClientView>();

	const code = $derived(page.params['code']);
	let ws: WebSocket | null | undefined;

	async function handleWebsocket() {
		if (code === undefined) {
			await goto(resolve('/', {}));
			return;
		}
		const result = await api.room.exists(code);

		if (!result.ok) {
			toast.error(m.error_generic());
			await goto(resolve('/', {}));
			return;
		}
		if (!result.value) {
			toast.error(m.room_not_found());
			await goto(resolve('/', {}));
			return;
		}

		ws = new WebSocket(`/ws/${code}`);
		room.send = (cmd) => {
			const s = readSession();
			if (s?.token) send({ kind: 'Authed', token: s.token, cmd });
		};
		ws.onopen = () => {
			const stored = readSession();
			if (stored?.room === code && stored?.token) {
				room.player = stored.player ?? null;
				send({ kind: 'Reconnect', token: stored.token });
			} else {
				nameOpen = true;
			}
		};
		ws.onmessage = (ev) => {
			console.log(ev);
			const serverMsg = JSON.parse(String(ev.data)) as ServerMessage;
			switch (serverMsg.kind) {
				case 'Joined':
					nameOpen = false;
					room.player = name;
					saveSession({ room: code, token: serverMsg.token, player: name });
					break;
				case 'Snapshot': {
					console.log(serverMsg);

					const prevPlayers = new Set(Object.keys(snapshot?.players ?? {}));
					const joined = Object.keys(serverMsg.players).filter((p) => !prevPlayers.has(p));
					joined.forEach((p) => toast.success(`${p} joined`));

					room.code = code;
					room.gamestate = serverMsg;
					snapshot = serverMsg;
					break;
				}
				case 'Error':
					toast.error(serverMsg.message);
					saveSession({ room: code });
					clearRoom();
					nameOpen = true;
					break;
				default: {
					console.error(`unhandled ServerMessage`);
				}
			}
		};
		ws.onerror = () => {
			toast.error(m.error_generic());
		};
		ws.onclose = () => {
			toast.error(m.error_generic());
			void goto(resolve('/', {}));
		};
	}

	onMount(() => {
		name = readSession()?.player ?? '';
		if (code === undefined) return;
		const existing = readSession();
		if (!existing || existing.room !== code) {
			saveSession({ room: code });
		}
		void handleWebsocket();
		return () => {
			if (ws) {
				ws.onmessage = ws.onerror = ws.onclose = null;
				ws.close();
			}
			clearRoom();
		};
	});

	function send(msg: ClientMessage) {
		ws?.send(JSON.stringify(msg));
	}

	function join() {
		send({ kind: 'Join', name });
	}
</script>

{#if snapshot}
	<GameStage view={snapshot} />
{/if}

<Dialog bind:open={nameOpen} title="Username">
	<form
		onsubmit={(e) => {
			e.preventDefault();
			join();
		}}
	>
		<TextInput bind:value={name} placeholder="Karl" />
		<Button disabled={!name}>{m.join()}</Button>
	</form>
</Dialog>
