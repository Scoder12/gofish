import { MutableRefObject, PropsWithChildren, useRef, useState } from "react";

function Center({ children }: PropsWithChildren<{}>) {
  return (
    <div className="w-full h-full flex justify-center items-center">
      {children}
    </div>
  );
}

enum GameStateId {
  Menu,
  Lobby,
}

type GameState = { type: GameStateId.Menu } | { type: GameStateId.Lobby };

type MaybeConnection = WebSocket | null;
type ConnectionRef = MutableRefObject<MaybeConnection>;

function Menu({ connection }: { connection: ConnectionRef }) {
  return (
    <Center>
      <div className="block">
        <div>
          <h3>Join Game</h3>
        </div>
        <div>
          <h3>Create Game</h3>
        </div>
      </div>
    </Center>
  );
}

export default function Home() {
  const [gameState, setGameState] = useState<GameState>({
    type: GameStateId.Menu,
  });
  const connection: ConnectionRef = useRef<MaybeConnection>(null);

  if (gameState.type == GameStateId.Menu) {
    return <Menu connection={connection} />;
  }
}
