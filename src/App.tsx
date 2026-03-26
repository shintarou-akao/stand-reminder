import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useAppStore, StateSnapshot } from "./store";
import SettingsView from "./SettingsView";
import "./App.css";

const windowLabel = getCurrentWebviewWindow().label;

function ModalView() {
  const { setFromBackend } = useAppStore();

  useEffect(() => {
    const unlisten = listen<StateSnapshot>("state-changed", (event) => {
      setFromBackend(event.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [setFromBackend]);

  return (
    <div className="overlay">
      <div className="card">
        <div className="icon-row">
          <span className="icon-emoji">🪑</span>
          <span className="icon-arrow">→</span>
          <span className="icon-emoji">🧍</span>
        </div>
        <div className="card-body">
          <h1>Time to Stand</h1>
          <p>You've been sitting too long.<br />Move around and refresh!</p>
        </div>
        <button className="btn-stand" onClick={() => invoke("stood_up")}>
          <span className="btn-icon">✓</span>
          I stood up!
        </button>
      </div>
    </div>
  );
}

function App() {
  if (windowLabel === "settings") return <SettingsView />;
  return <ModalView />;
}

export default App;
