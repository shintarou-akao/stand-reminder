import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

type ReminderMode = "interval" | "specificTimes";

interface Settings {
  reminderMode: ReminderMode;
  remindIntervalMins: number;
  specificTimes: string[];
}

function SettingsView() {
  const [mode, setMode] = useState<ReminderMode>("interval");
  const [mins, setMins] = useState(25);
  const [times, setTimes] = useState<string[]>([]);

  useEffect(() => {
    invoke<Settings>("get_settings").then((s) => {
      setMode(s.reminderMode);
      setMins(s.remindIntervalMins);
      setTimes(s.specificTimes);
    }).catch(console.error);
  }, []);

  const clamp = (v: number) => Math.max(1, Math.min(999, v));

  const addTime = () => setTimes((prev) => [...prev, "09:00"]);

  const updateTime = (i: number, val: string) => {
    const next = [...times];
    next[i] = val;
    setTimes(next);
  };

  const removeTime = (i: number) =>
    setTimes((prev) => prev.filter((_, idx) => idx !== i));

  const handleSave = async () => {
    await invoke("save_settings", {
      settings: { reminderMode: mode, remindIntervalMins: mins, specificTimes: [...times].sort() },
    });
    getCurrentWebviewWindow().close();
  };

  return (
    <div className="s-root">
      <div className="s-body">
        <p className="s-title">リマインド設定</p>

        <div className={`s-group${mode !== "interval" ? " s-inactive" : ""}`}>
          <div className="s-row" onClick={() => setMode("interval")} style={{ cursor: "pointer" }}>
            <input type="radio" className="s-radio" readOnly checked={mode === "interval"} />
            <span className="s-icon">⏰</span>
            <span className="s-label">通知間隔</span>
            <div className="stepper" onClick={(e) => e.stopPropagation()}>
              <button className="stepper-btn" disabled={mode !== "interval"} onClick={() => setMins(clamp(mins - 1))}>−</button>
              <input
                className="stepper-input"
                type="number"
                min={1}
                max={999}
                value={mins}
                disabled={mode !== "interval"}
                onChange={(e) => setMins(clamp(parseInt(e.target.value) || 1))}
              />
              <span className="stepper-unit">分</span>
              <button className="stepper-btn" disabled={mode !== "interval"} onClick={() => setMins(clamp(mins + 1))}>＋</button>
            </div>
          </div>
        </div>

        <div className={`s-group${mode !== "specificTimes" ? " s-inactive" : ""}`}>
          <div className="s-row" onClick={() => setMode("specificTimes")} style={{ cursor: "pointer" }}>
            <input type="radio" className="s-radio" readOnly checked={mode === "specificTimes"} />
            <span className="s-icon">🕐</span>
            <span className="s-label">時刻指定</span>
          </div>
          {mode === "specificTimes" && (
            <>
              {times.map((t, i) => (
                <div key={i} className="s-row s-divider">
                  <input
                    className="time-input"
                    type="time"
                    value={t}
                    onChange={(e) => updateTime(i, e.target.value)}
                  />
                  <button className="time-remove" onClick={() => removeTime(i)}>×</button>
                </div>
              ))}
              {times.length === 0 && <div className="s-empty">時刻を追加してください</div>}
              <div className="s-row s-add-row">
                <button className="s-add-btn" onClick={addTime}>＋ 時刻を追加</button>
              </div>
            </>
          )}
        </div>
      </div>

      <div className="s-footer">
        <button className="s-btn-cancel" onClick={() => getCurrentWebviewWindow().close()}>
          キャンセル
        </button>
        <button className="s-btn-save" onClick={handleSave}>
          保存
        </button>
      </div>
    </div>
  );
}

export default SettingsView;
