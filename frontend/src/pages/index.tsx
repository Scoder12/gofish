import { Field, Formik } from "formik";
import {
  ButtonHTMLAttributes,
  MutableRefObject,
  PropsWithChildren,
  useRef,
  useState,
} from "react";

function Center({ children }: PropsWithChildren<{}>) {
  return (
    <div className="w-full h-full flex justify-center items-center">
      {children}
    </div>
  );
}

enum GameStateId {
  Menu,
  Connecting,
  Lobby,
}

type GameState =
  | { type: GameStateId.Menu }
  | { type: GameStateId.Connecting }
  | { type: GameStateId.Lobby };

type MaybeConnection = WebSocket | null;
type ConnectionRef = MutableRefObject<MaybeConnection>;

function Button({
  children,
  className,
  ...props
}: PropsWithChildren<ButtonHTMLAttributes<HTMLButtonElement>>) {
  return (
    <button
      className={
        "border border-neutral-800 rounded-sm bg-neutral-200 hover:bg-neutral-300 " +
        "disabled:bg-neutral-200 disabled:opacity-50 p-2 text-xl " +
        (className ?? "")
      }
      {...props}
    >
      {children}
    </button>
  );
}

function JoinGameForm() {
  return (
    <Formik initialValues={{ pin: "" }} onSubmit={console.log}>
      {({ isSubmitting }) => <div className="flex"></div>}
    </Formik>
  );
}

function Menu({ connection }: { connection: ConnectionRef }) {
  return (
    <Center>
      <Formik
        initialValues={{ name: "", pin: "", create: false }}
        validate={({ name }) => {
          if (!name) {
            return { name: "Name is required" };
          }
          return {};
        }}
        onSubmit={({ name, pin, create }, { setSubmitting }) => {
          console.log({ name, pin, create });
          setTimeout(() => setSubmitting(false), 1000);
        }}
      >
        {({ isSubmitting, setValues, submitForm, values, errors }) => (
          <div className="block">
            <div className="block">
              <h3>Enter Name</h3>
              <div
                className={
                  "text-red-500 font-semibold " +
                  (errors.name ? "" : "invisible")
                }
              >
                {errors.name || "A"}
              </div>
              <Field type="text" name="name" className="text-3xl" size="15" />
            </div>
            <hr className="my-3" />
            <div>
              <h3>Existing Game</h3>
              <div className="flex w-full">
                <Field
                  type="number"
                  name="pin"
                  className="text-3xl mr-1 flex-grow"
                  size="10"
                />
                <Button
                  disabled={isSubmitting}
                  onClick={() => {
                    setValues({ ...values, create: false });
                    submitForm();
                  }}
                >
                  JOIN
                </Button>
              </div>
            </div>
            <div>
              <h3>New Game</h3>
              <Button
                className="float-right"
                disabled={isSubmitting}
                onClick={() => {
                  setValues({ ...values, create: true });
                  submitForm();
                }}
              >
                Create Game
              </Button>
            </div>
          </div>
        )}
      </Formik>
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
