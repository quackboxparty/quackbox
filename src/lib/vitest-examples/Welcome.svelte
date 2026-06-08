<script lang="ts">
	import { greet } from './greet';

	let { guest = 'Vitest', host = 'Sveltekit' }: { guest?: string; host?: string } = $props();
</script>

<h1>{greet(host)}</h1>
<p>{greet(guest)}</p>
