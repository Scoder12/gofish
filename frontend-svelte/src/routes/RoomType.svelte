<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import Button from '../lib/Button.svelte';

	const dispatch = createEventDispatcher();

	export let name: string;
	let pin: number;
</script>

<div>
	<h3>Hello, {name}</h3>
	<div>
		<h4>Existing Game</h4>
		<div class="flex w-full">
			<input type="number" name="pin" class="text-3xl mr-1 flex-grow" size="10" bind:value={pin} />
			<Button on:click={() => dispatch('next', { type: 'join', pin })}>JOIN</Button>
		</div>
	</div>
	<div class="block">
		<h4>New Game</h4>
		<Button class="float-right" on:click={() => dispatch('next', { type: 'create' })}
			>Create Game</Button
		>
	</div>
</div>
