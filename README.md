# Stand Reminder

長時間座り続けるのを防ぐ、macOS用のメニューバー常駐アプリです。

## 機能

- **メニューバー常駐** — Dock に表示されず、常にバックグラウンドで動作
- **2つのリマインドモード**
  - **通知間隔モード** — 指定した分数ごとにリマインド（デフォルト25分）
  - **時刻指定モード** — 決まった時刻にリマインド（複数設定可能）
- **モーダル通知** — 他のウィンドウの上に表示され、「立った」ボタンで解除
- **スリープ検知** — Macのスリープ復帰後にタイマーをリセット
- **マルチモニター対応** — カーソルのあるモニターの中央に通知を表示
- **ダークモード対応**

## 動作環境

- macOS（Apple Silicon / Intel）

## インストール

ビルド済みバイナリは提供していません。ソースからビルドしてください。

## ビルド手順

### 前提条件

- [Rust](https://www.rust-lang.org/tools/install)（最新の stable）
- [Node.js](https://nodejs.org/) v18以上
- [pnpm](https://pnpm.io/)

```bash
# pnpm がない場合
npm install -g pnpm
```

### ビルド

```bash
git clone https://github.com/yourusername/stand-reminder.git
cd stand-reminder
pnpm install
pnpm tauri build
```

ビルドされたアプリは `src-tauri/target/release/bundle/macos/Stand Reminder.app` に生成されます。

### 開発

```bash
pnpm tauri dev
```

## 技術スタック

- **フロントエンド** — React 19 + TypeScript + Zustand + Vite
- **バックエンド** — Rust + Tauri v2 + tokio

## ライセンス

MIT License — 詳細は [LICENSE](LICENSE) を参照してください。
