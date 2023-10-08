import createActorStoreAndActions from "@re-actor/store"
import { canisterId, createActor } from "../../declarations/payment_backend"

const [store, actions] = createActorStoreAndActions(() =>
  createActor(canisterId)
)

store.subscribe((state) => {
  console.log("state", state)
})

function ConnectButton() {
  return <button onClick={actions.connect}>Connect</button>
}

function App() {
  const { connected } = store.use()
  return <h1>Hello, world!</h1>
}

const container = document.getElementById("root")
const root = ReactDOM.createRoot(container)
root.render(<App />)
