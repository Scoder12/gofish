// automatically generated by the FlatBuffers compiler, do not modify

import { ErrorS } from '../go-fish/error-s.js';
import { GameCreationResponseS } from '../go-fish/game-creation-response-s.js';


export enum Smsg {
  NONE = 0,
  ErrorS = 1,
  GameCreationResponseS = 2
}

export function unionToSmsg(
  type: Smsg,
  accessor: (obj:ErrorS|GameCreationResponseS) => ErrorS|GameCreationResponseS|null
): ErrorS|GameCreationResponseS|null {
  switch(Smsg[type]) {
    case 'NONE': return null; 
    case 'ErrorS': return accessor(new ErrorS())! as ErrorS;
    case 'GameCreationResponseS': return accessor(new GameCreationResponseS())! as GameCreationResponseS;
    default: return null;
  }
}

export function unionListToSmsg(
  type: Smsg, 
  accessor: (index: number, obj:ErrorS|GameCreationResponseS) => ErrorS|GameCreationResponseS|null, 
  index: number
): ErrorS|GameCreationResponseS|null {
  switch(Smsg[type]) {
    case 'NONE': return null; 
    case 'ErrorS': return accessor(index, new ErrorS())! as ErrorS;
    case 'GameCreationResponseS': return accessor(index, new GameCreationResponseS())! as GameCreationResponseS;
    default: return null;
  }
}
