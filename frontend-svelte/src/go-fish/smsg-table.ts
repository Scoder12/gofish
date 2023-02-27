// automatically generated by the FlatBuffers compiler, do not modify

import * as flatbuffers from 'flatbuffers';

import { Smsg, unionToSmsg, unionListToSmsg } from '../go-fish/smsg.js';


export class SmsgTable {
  bb: flatbuffers.ByteBuffer|null = null;
  bb_pos = 0;
  __init(i:number, bb:flatbuffers.ByteBuffer):SmsgTable {
  this.bb_pos = i;
  this.bb = bb;
  return this;
}

static getRootAsSmsgTable(bb:flatbuffers.ByteBuffer, obj?:SmsgTable):SmsgTable {
  return (obj || new SmsgTable()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
}

static getSizePrefixedRootAsSmsgTable(bb:flatbuffers.ByteBuffer, obj?:SmsgTable):SmsgTable {
  bb.setPosition(bb.position() + flatbuffers.SIZE_PREFIX_LENGTH);
  return (obj || new SmsgTable()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
}

msgType():Smsg {
  const offset = this.bb!.__offset(this.bb_pos, 4);
  return offset ? this.bb!.readUint8(this.bb_pos + offset) : Smsg.NONE;
}

msg<T extends flatbuffers.Table>(obj:any):any|null {
  const offset = this.bb!.__offset(this.bb_pos, 6);
  return offset ? this.bb!.__union(obj, this.bb_pos + offset) : null;
}

static startSmsgTable(builder:flatbuffers.Builder) {
  builder.startObject(2);
}

static addMsgType(builder:flatbuffers.Builder, msgType:Smsg) {
  builder.addFieldInt8(0, msgType, Smsg.NONE);
}

static addMsg(builder:flatbuffers.Builder, msgOffset:flatbuffers.Offset) {
  builder.addFieldOffset(1, msgOffset, 0);
}

static endSmsgTable(builder:flatbuffers.Builder):flatbuffers.Offset {
  const offset = builder.endObject();
  return offset;
}

static finishSmsgTableBuffer(builder:flatbuffers.Builder, offset:flatbuffers.Offset) {
  builder.finish(offset);
}

static finishSizePrefixedSmsgTableBuffer(builder:flatbuffers.Builder, offset:flatbuffers.Offset) {
  builder.finish(offset, undefined, true);
}

static createSmsgTable(builder:flatbuffers.Builder, msgType:Smsg, msgOffset:flatbuffers.Offset):flatbuffers.Offset {
  SmsgTable.startSmsgTable(builder);
  SmsgTable.addMsgType(builder, msgType);
  SmsgTable.addMsg(builder, msgOffset);
  return SmsgTable.endSmsgTable(builder);
}
}
