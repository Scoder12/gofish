<script lang="ts">
	import Button from '../lib/Button.svelte';
	import Center from '../lib/Center.svelte';

	import { form, field } from 'svelte-forms';
	import { required } from 'svelte-forms/validators';

	const name = field('name', '', [required()]);
	const pin = field('pin', '', []);
	const myForm = form(name);

	let isSubmitting = false;
	let formDisabled = false;
	$: formDisabled = !$myForm.valid || isSubmitting;

	function submit() {
		isSubmitting = true;
	}
</script>

<Center>
	<div class="block">
		<div class="block">
			<h3>Enter Name</h3>
			<div
				class={'text-red-500 font-semibold ' +
					($myForm.hasError('name.required') ? 'invisible' : '')}
			>
				Name is required
			</div>
			<input type="text" name="name" class="text-3xl" size="15" bind:value={$name.value} />
		</div>
		<hr class="my-3" />
		<div>
			<h3>Existing Game</h3>
			<div class="flex w-full">
				<input type="number" name="pin" class="text-3xl mr-1 flex-grow" size="10" value={pin} />
				<Button disabled={formDisabled} on:click={submit}>JOIN</Button>
			</div>
		</div>
		<div>
			<h3>New Game</h3>
			<Button class="float-right" disabled={formDisabled} on:click={submit}>Create Game</Button>
		</div>
	</div>
</Center>
