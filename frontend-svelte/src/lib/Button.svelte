<button
	{...$$props}
	class={'border border-neutral-800 rounded-sm bg-neutral-200 hover:bg-neutral-300 ' +
		'disabled:bg-neutral-200 disabled:opacity-50 p-2 text-xl ' +
		($$props.class ?? '')}
	on:click
>
	<slot />
</button>
