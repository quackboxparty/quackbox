<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { api } from '$lib/api';
	import type { ClientMessage, ClientView, ServerMessage } from '$lib/bindings/Protocol';
	import Dialog from '$lib/components/Dialog.svelte';
	import TextInput from '$lib/components/TextInput.svelte';
	import { m } from '$lib/paraglide/messages';
	import { clearSession, readSession, saveSession } from '$lib/session';
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
		ws.onopen = () => {
			const stored = readSession();
			if (stored?.room === code) {
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
					saveSession({ room: code, token: serverMsg.token });
					break;
				case 'Snapshot': {
					console.log(serverMsg);

					const prevPlayers = snapshot?.players ?? [];
					const newPlayers = serverMsg.players;
					const joined = newPlayers.filter((p) => !prevPlayers.includes(p));
					joined.forEach((p) => toast.success(`${p} joined`));

					snapshot = serverMsg;
					break;
				}
				case 'Error':
					toast.error(serverMsg.message);
					clearSession();
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
		void handleWebsocket();
		return () => ws?.close();
	});

	function send(msg: ClientMessage) {
		ws?.send(JSON.stringify(msg));
	}

	function join() {
		send({ kind: 'Join', name });
	}
</script>

{#if snapshot}
	<h2>Players:</h2>
	{#each snapshot.players as player (player)}
		<div class="card">
			<h3>{player}</h3>
		</div>
	{/each}
{/if}

<Dialog bind:open={nameOpen} title="Username">
	<form onsubmit={join}>
		<TextInput bind:value={name} placeholder="Karl" />
	</form>
</Dialog>
