<script lang="ts">
	import Center from '../lib/Center.svelte';
	import * as flatbuffers from 'flatbuffers';

	import Name from './Name.svelte';
	import RoomType from './RoomType.svelte';
	import {
		Cmsg,
		CmsgTable,
		Create,
		ErrorS,
		GameCreationResponseS,
		GameRef,
		IdentifyC,
		Join,
		Smsg,
		SmsgTable
	} from '../go-fish';
	import { env } from '$env/dynamic/public';

	enum Stage {
		Name,
		RoomType,
		Connecting
	}

	let stage = Stage.Name;
	let name = '';

	function gameFromDetails(fbb: flatbuffers.Builder, detail: any): [GameRef, number] {
		if (detail.type == 'create') {
			return [GameRef.Create, Create.createCreate(fbb)];
		}
		if (detail.type == 'join') {
			return [GameRef.Join, Join.createJoin(fbb, detail.pin)];
		}
		throw new Error('Invalid game type');
	}

	function identifyMsg(detail: any): Uint8Array {
		const fbb = new flatbuffers.Builder(1);
		const nameOffset = fbb.createString(name);
		const [gameType, gameOffset] = gameFromDetails(fbb, detail);
		const identifyOffset = IdentifyC.createIdentifyC(fbb, nameOffset, gameType, gameOffset);
		const msgTableOffset = CmsgTable.createCmsgTable(fbb, Cmsg.IdentifyC, identifyOffset);
		fbb.finish(msgTableOffset, undefined);
		return fbb.asUint8Array();
	}

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

	function unpackMessage<T>(msg: ArrayBuffer, typ: Smsg, obj: T): T {
		const bb = new flatbuffers.ByteBuffer(new Uint8Array(msg));
		const smsg = SmsgTable.getRootAsSmsgTable(bb);
		if (smsg.msgType() == Smsg.ErrorS) {
			const error: ErrorS = smsg.msg(new ErrorS());
			throw new Error(`Error from server: ${error.error()}`);
		}
		if (smsg.msgType() != typ) {
			throw new Error(`Unexpected message type ${smsg.msgType()}, expected ${typ}`);
		}
		return smsg.msg(obj);
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
			const r = unpackMessage(resp.data, Smsg.GameCreationResponseS, new GameCreationResponseS());
			console.log(r.id());
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
	{/if}
</Center>
