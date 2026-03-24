use tauri::{AppHandle, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};

/// カーソルがあるモニター上にウィンドウを中央配置する（物理ピクセルで計算）
fn move_to_cursor_monitor(window: &tauri::WebviewWindow, win_w: f64, win_h: f64) {
    let app = window.app_handle();

    let Ok(cursor) = app.cursor_position() else {
        let _ = window.center();
        return;
    };
    let Ok(monitors) = app.available_monitors() else {
        let _ = window.center();
        return;
    };

    // cursor_position() は物理ピクセル、Monitor::position/size も物理ピクセル
    let monitor = monitors.into_iter().find(|m| {
        let p = m.position();
        let s = m.size();
        cursor.x >= p.x as f64
            && cursor.x < p.x as f64 + s.width as f64
            && cursor.y >= p.y as f64
            && cursor.y < p.y as f64 + s.height as f64
    });

    let _ = if let Some(m) = monitor {
        let scale = m.scale_factor();
        let p = m.position();
        let s = m.size();

        // 論理サイズ → 物理サイズ
        let win_w_phys = win_w * scale;
        let win_h_phys = win_h * scale;

        // モニター上で中央になる物理座標
        let x = p.x as f64 + (s.width as f64 - win_w_phys) / 2.0;
        let y = p.y as f64 + (s.height as f64 - win_h_phys) / 2.0;

        window.set_position(PhysicalPosition::new(x as i32, y as i32))
    } else {
        window.center()
    };
}

pub fn show_modal(app: &AppHandle) -> tauri::Result<()> {
    if app.get_webview_window("modal").is_some() {
        return Ok(());
    }

    let (w, h) = (420.0_f64, 300.0_f64);

    let window = WebviewWindowBuilder::new(app, "modal", WebviewUrl::App("index.html".into()))
        .title("Stand Reminder")
        .inner_size(w, h)
        .always_on_top(true)
        .closable(false)
        .minimizable(false)
        .maximizable(false)
        .resizable(false)
        .decorations(false)
        .focused(true)
        .skip_taskbar(true)
        .build()?;

    move_to_cursor_monitor(&window, w, h);

    Ok(())
}

pub fn hide_modal(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("modal") {
        let _ = window.destroy();
    }
}

pub fn show_settings(app: &AppHandle) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.set_focus();
        return Ok(());
    }

    let (w, h) = (360.0_f64, 420.0_f64);

    let window = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index.html".into()))
        .title("Stand Reminder - 設定")
        .inner_size(w, h)
        .resizable(false)
        .build()?;

    move_to_cursor_monitor(&window, w, h);

    Ok(())
}
