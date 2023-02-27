<script lang="ts">
	import Button from '../lib/Button.svelte';

	export let name: string;
</script>

<div>
	<h3>Hello, {name}</h3>
	<div>
		<h4>Existing Game</h4>
		<div class="flex w-full">
			<input type="number" name="pin" class="text-3xl mr-1 flex-grow" size="10" />
			<Button>JOIN</Button>
		</div>
	</div>
	<div class="block">
		<h4>New Game</h4>
		<Button class="float-right">Create Game</Button>
	</div>
</div>
