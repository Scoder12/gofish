<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { createForm } from 'svelte-forms-lib';
	import Button from '../lib/Button.svelte';

	const dispatch = createEventDispatcher();

	const { form, errors, handleChange, handleSubmit } = createForm({
		initialValues: {
			name: ''
		},
		validate: (values) => {
			let errors: any = {};
			if (values.name === '') {
				errors.name = 'Name is required';
			}
			return errors;
		},
		onSubmit: ({ name }) => {
			dispatch('next', { name });
		}
	});
</script>

<form class="block" on:submit|preventDefault={handleSubmit}>
	<h3>Enter Name</h3>
	<div class={'text-red-500 font-semibold ' + ($errors.name ? '' : 'invisible')}>
		{$errors.name || 'A'}
	</div>
	<input
		type="text"
		name="name"
		class="text-3xl block mb-1"
		size="15"
		on:change={handleChange}
		bind:value={$form.name}
	/>
	<Button type="submit" class="float-right">Next</Button>
</form>
