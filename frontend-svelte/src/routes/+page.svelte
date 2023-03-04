<script lang="ts">
	import Center from '../lib/Center.svelte';

	import Name from './Name.svelte';
	import RoomType from './RoomType.svelte';
	import { env } from '$env/dynamic/public';
	import { Msg as C2SMsg, GameRef, Identify } from '../c2s.capnp';
	import * as capnp from 'capnp-ts';

	enum Stage {
		Name,
		RoomType,
		Connecting
	}

	let stage = Stage.Name;
	let name = '';

	let ws: WebSocket | null = null;

	function nextMessage(socket: WebSocket): Promise<MessageEvent<ArrayBuffer>> {
		return new Promise((resolve, reject) => {
			socket.addEventListener('close', reject);
			const listener = (msg: MessageEvent<any>) => {
				if (!(msg.data instanceof ArrayBuffer)) {
					// ignore text frames
					return;
				}
				socket.removeEventListener('close', reject);
				socket.removeEventListener('message', listener);
				resolve(msg);
			};
			socket.addEventListener('message', listener);
		});
	}

	function identifyMsg(detail: any): ArrayBuffer {
		const buf = new capnp.Message();
		const msg = buf.initRoot(C2SMsg as any) as unknown as C2SMsg;
		const id = msg.initIdentify();
		id.setName(name);
		const game = id.initGame();
		if (detail.type == 'create') {
			game.setCreate();
		} else if (detail.type == 'join') {
			game.setJoin(detail.pin);
		} else {
			throw new Error('Unexpected join type');
		}
		return buf.toArrayBuffer();
	}

	function connect(detail: any) {
		const msg = identifyMsg(detail);
		const wsUrl = env.PUBLIC_WS_URL;
		if (!wsUrl) throw new Error('misconfigured');
		ws = new WebSocket(wsUrl);
		ws.binaryType = 'arraybuffer';
		ws.onclose = () => {
			ws = null;
			throw new Error('disconnected');
		};
		ws.onopen = async () => {
			ws!.send(msg);
			const resp = await nextMessage(ws!);
		};
	}
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
		<RoomType
			{name}
			on:next={(evt) => {
				stage = Stage.Connecting;
				connect(evt.detail);
			}}
		/>
	{:else if stage == Stage.Connecting}
		<p>Connecting...</p>
	{/if}
</Center>
