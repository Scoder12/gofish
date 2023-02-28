@0xa647b63e43643d15;

struct GameRef {
  union {
    create @0 :Void;
    join @1 :UInt32;
  }
}

struct Identify {
  name @0 :Text;
  game @1 :GameRef;
}

struct Msg {
  union {
    identify @0 :Identify;
    startGame @1 :Void;
  }
}
