<script lang="ts">
	import Button from '../lib/Button.svelte';
	import Center from '../lib/Center.svelte';

	let name = '';
	let pin: number | null = null;
	let errors = { name: '' };
	let isSubmitting = false;
</script>

<Center>
	<div class="block">
		<div class="block">
			<h3>Enter Name</h3>
			<div class={'text-red-500 font-semibold ' + (errors.name ? '' : 'invisible')}>
				{errors.name || 'A'}
			</div>
			<input type="text" name="name" class="text-3xl" size="15" value={name} />
		</div>
		<hr class="my-3" />
		<div>
			<h3>Existing Game</h3>
			<div class="flex w-full">
				<input type="number" name="pin" class="text-3xl mr-1 flex-grow" size="10" value={pin} />
				<Button disabled={isSubmitting}>JOIN</Button>
			</div>
		</div>
		<div>
			<h3>New Game</h3>
			<Button class="float-right" disabled={isSubmitting}>Create Game</Button>
		</div>
	</div>
</Center>
