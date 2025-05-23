import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface Ok<T> {
  Ok: T,
}

interface Error {
  Err: string,
}

type Result<T> = Ok<T> | Error

enum GitType{
  GitHub = "GitHub",
  GitLab = "GitLab",
}

interface Setting {
  git_type: GitType,
  owner: String,
  repo: String,
  order: number,
}

interface Fetched {
  setting: Setting,
  stars: number,
}
 
// First time load data
const data: Result<Fetched>[] = await invoke("read");

function App() {
  const [greetMsg, setGreetMsg] = useState(0);
  const [owner, setOwner] = useState("");
  const [repo, setRepo] = useState("");

  const [settings, setSettings] = useState(data);

  async function add() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    let setting: Setting = {
      git_type: GitType.GitHub,
      owner: owner,
      repo: repo,
      order: 0
    }
    setGreetMsg(await invoke("create", { setting }));
    await invoke("set_current", { setting });
  }

  return (
    <main className="container">
      <h1>Welcome to Tauri + React</h1>

      <div className="row">
        <a href="https://vite.dev" target="_blank">
          <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <p>Click on the Tauri, Vite, and React logos to learn more.</p>

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          add();
        }}
      >
        <input
          id="owner-input"
          onChange={(e) => setOwner(e.currentTarget.value)}
          placeholder="Enter a owner..."
        />
        <input
          id="repo-input"
          onChange={(e) => setRepo(e.currentTarget.value)}
          placeholder="Enter a repo..."
        />
        <button type="submit">Greet</button>
      </form>
      <p>{greetMsg}</p>
    </main>
  );
}

export default App;
