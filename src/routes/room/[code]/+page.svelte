<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { api } from '$lib/api';
	import type { ClientMessage, ClientView, ServerMessage } from '$lib/bindings/Protocol';
	import Button from '$lib/components/Button.svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import TextInput from '$lib/components/TextInput.svelte';
	import { m } from '$lib/paraglide/messages';
	import { clearSession, readSession, saveSession } from '$lib/session';
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
			if (s) send({ kind: 'Authed', token: s.token, cmd });
		};
		ws.onopen = () => {
			const stored = readSession();
			if (stored?.room === code) {
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
					clearSession();
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
	}

	onMount(() => {
		name = readSession()?.player ?? '';
		void handleWebsocket();
		return () => {
			ws?.close();
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

<div class="hero">
	{#if snapshot}
		<h2>Players:</h2>
		{#each Object.keys(snapshot.players) as player (player)}
			<div class="card">
				<h3>{player}</h3>
			</div>
		{/each}
	{/if}
</div>

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

<style>
	.hero {
		min-height: 100%;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-6);
		text-align: center;
		padding: var(--space-12) var(--space-6);
	}
</style>
