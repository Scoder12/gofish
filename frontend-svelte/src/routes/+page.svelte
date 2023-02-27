<script lang="ts">
	import Center from '../lib/Center.svelte';

	import Name from './Name.svelte';
	import RoomType from './RoomType.svelte';

	enum Stage {
		Name,
		RoomType
	}

	let stage = Stage.Name;
	let name = '';
</script>

<Center>
	{#if stage == Stage.Name}
		<Name
			on:next={(evt) => {
				name = evt.detail.name;
				stage = Stage.RoomType;
			}}
		/>
	{:else if stage == Stage.RoomType}
		<RoomType {name} />
	{/if}
</Center>
