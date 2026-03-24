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
        <div className="icon">🧍</div>
        <h1>立ってください</h1>
        <p>長時間座っています。立ち上がりましょう！</p>
        <div className="button-group">
          <button className="btn-primary" onClick={() => invoke("stood_up")}>
            立った
          </button>
        </div>
      </div>
    </div>
  );
}

function App() {
  if (windowLabel === "settings") return <SettingsView />;
  return <ModalView />;
}

export default App;
