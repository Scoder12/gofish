<script lang="ts">
	import Button from '../lib/Button.svelte';
	import Center from '../lib/Center.svelte';

	import { createForm } from 'svelte-forms-lib';

	const { form, errors, state, handleChange, handleSubmit } = createForm({
		initialValues: {
			name: '',
			pin: ''
		},
		validate: (values) => {
			let errors: any = {};
			if (values.name === '') {
				errors.name = 'Name is required';
			}
			return errors;
		},
		onSubmit: (values) => {
			alert(JSON.stringify(values));
		}
	});

	let formDisabled = false;
	function submit() {}
</script>

<Center>
	<form class="block" on:submit={handleSubmit}>
		<div class="block">
			<h3>Enter Name</h3>
			<div class={'text-red-500 font-semibold ' + ($errors.name ? '' : 'invisible')}>
				{$errors.name || 'A'}
			</div>
			<input
				type="text"
				name="name"
				class="text-3xl"
				size="15"
				on:change={handleChange}
				bind:value={$form.name}
			/>
		</div>
		<hr class="my-3" />
		<div>
			<h3>Existing Game</h3>
			<div class="flex w-full">
				<input
					type="number"
					name="pin"
					class="text-3xl mr-1 flex-grow"
					size="10"
					on:change={handleChange}
					bind:value={$form.pin}
				/>
				<Button disabled={formDisabled} type="submit">JOIN</Button>
			</div>
		</div>
		<div>
			<h3>New Game</h3>
			<Button class="float-right" disabled={formDisabled} type="submit">Create Game</Button>
		</div>
	</form>
</Center>
