# 座りすぎ防止アプリ（Stand Reminder）実装計画

## Context

長時間座り作業を防止するmacOSメニューバー常駐アプリを新規作成する。Tauri v2 + React + TypeScript + Zustandで構築。ディレクトリは空の状態から開始し、**Rustが未インストール**のため環境構築から始める。

## アーキテクチャ

**Rust（バックエンド）**: タイマーロジック、状態管理、トレイメニュー、モーダルウィンドウ制御、スリープ検知
**React（フロントエンド）**: モーダル通知UIのみ（メニューバーはRust側で完結）

Rustが全状態を保持し、Reactはモーダル描画専用の薄いレイヤー。

## ファイル構成

```
stand-reminder/
├── src/                          # React
│   ├── App.tsx                   # モーダルUI
│   ├── App.css                   # スタイル
│   ├── main.tsx
│   └── store.ts                  # Zustand（バックエンド状態のミラー）
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/default.json
│   ├── icons/
│   └── src/
│       ├── lib.rs                # Tauriセットアップ、コマンド登録
│       ├── main.rs
│       ├── state.rs              # AppState構造体、モード遷移
│       ├── timer.rs              # tokioベースの1秒tickタイマー
│       ├── tray.rs               # トレイメニュー構築・更新
│       ├── modal.rs              # モーダルウィンドウ作成・破棄
│       └── sleep_detect.rs       # macOSスリープ復帰検知
```

## 実装フェーズ

### Phase 0: 環境構築
- [ ] Rustインストール（`rustup`）
- [ ] `pnpm create tauri-app stand-reminder`でプロジェクトスキャフォールド（React + TypeScript）
- [ ] `pnpm add zustand`
- [ ] `Cargo.toml`に依存追加: `tokio`(time,sync), `serde`, `serde_json`
- [ ] `tauri.conf.json`設定: `"windows": []`（起動時ウィンドウなし）、トレイアイコン、`LSUIElement`（Dock非表示）

### Phase 1: トレイ + モード管理（最重要）
- [ ] `state.rs`: `AppState`構造体（mode, timer_remaining_secs, timer_state, snooze_used, enabled）
- [ ] `tray.rs`: トレイメニュー構築（「座り中/立ち中」表示、モード切替、ON/OFF、終了）
- [ ] `lib.rs`: Tauriセットアップ、状態管理、トレイ初期化
- [ ] 検証: トレイアイコン表示、メニュー操作でモード切替

### Phase 2: タイマー
- [ ] `timer.rs`: `tokio::spawn`で1秒tickループ
  - 座りモード: 25分カウントダウン → 0で`timer-expired`イベント発火
  - 立ちモード: 30分後に自動で座りモードへ復帰
- [ ] `Instant`ベースで経過時間を計算（App Nap対策）
- [ ] 検証: テスト用に短い時間で動作確認

### Phase 3: モーダル通知
- [ ] `modal.rs`: `WebviewWindowBuilder`でモーダル作成
  - `always_on_top(true)`, `closable(false)`, `decorations(false)`, `resizable(false)`
  - `close_requested`イベントで閉じを阻止（Cmd+W対策）
- [ ] `App.tsx`: 「立ってください」UI、「立った」「スヌーズ（5分）」ボタン
- [ ] `App.css`: ダークモード対応、ミニマルデザイン、400x250ウィンドウ
- [ ] Tauriコマンド: `stood_up`（立ちモード遷移+モーダル閉じ）、`snooze`
- [ ] `store.ts`: Zustandでバックエンド状態をミラー

### Phase 4: スヌーズ
- [ ] スヌーズ1回制限ロジック（`snooze_used`フラグ）
- [ ] 5分後にモーダル再表示
- [ ] UIでスヌーズボタンを無効化表示

### Phase 5: スリープ復帰対応
- [ ] `sleep_detect.rs`: NSWorkspace通知でスリープ復帰検知（`objc2`クレート）
  - フォールバック: wall-clock vs monotonic時間の差分で検知
- [ ] 復帰時にタイマーリセット

### Phase 6: 仕上げ
- [ ] トレイアイコン（座り/立ち別アイコン、macOSテンプレートイメージ）
- [ ] トレイタイトルに残り時間表示
- [ ] エラーハンドリング
- [ ] `pnpm tauri build`でリリースビルド

## 主要Tauri v2 API

| 機能 | API |
|------|-----|
| システムトレイ | `tauri::tray::TrayIconBuilder` |
| トレイメニュー | `tauri::menu::{Menu, MenuItem}` |
| 状態管理 | `app.manage()`, `tauri::State<>` |
| Rust→JSイベント | `app.emit()` |
| コマンド | `#[tauri::command]`, `invoke()` |
| ウィンドウ作成 | `WebviewWindowBuilder::new()` |
| Dock非表示 | `LSUIElement` in Info.plist |

## 注意点

1. **LSUIElement**: Tauri v2に専用設定なし → `bundle.macOS.infoPlistString`で追加
2. **Cmd+W対策**: `close_requested`イベントで`prevent_close()`
3. **タイマー精度**: `Instant`ベースでApp Nap耐性を確保
4. **テスト用定数**: タイマー値は`state.rs`に定数として抽出し開発時に短縮可能に

## 検証方法

1. `pnpm tauri dev`で起動、Dockに表示されないことを確認
2. トレイメニューからモード切替が動作すること
3. タイマー（テスト用短縮値）終了後にモーダル表示
4. モーダルがEsc/Cmd+W/外クリックで閉じないこと
5. 「立った」でモーダル閉じ→立ちモード遷移
6. スヌーズ1回のみ→5分後再表示→ボタン無効
7. スリープ復帰でタイマーリセット
