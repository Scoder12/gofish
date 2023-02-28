@0xc668f14b39d58553;

struct Error {
  msg @0 :Text;
}

struct GameCreated {
  id @0 :UInt32;
}

struct Msg {
  union {
    error @0 :Error;
    gameCreated @1 :GameCreated;
  }
}
