# Better Clipboard

軽量クリップボードマネージャ（Windows）。

## 技術スタック

- **フレームワーク**: Tauri 2 (Rust + Web UI)
- **フロントエンド**: HTML / CSS / TypeScript (Vite)
- **バックエンド**: Rust
- **DB**: SQLite (rusqlite)

## 機能

### 実装済み

- **グローバルホットキー** (`Alt+C`) でオーバーレイ表示
- **クリップボード監視** — ポーリング方式（テキスト UTF-16）
- **オーバーレイ UI** — ダークテーマ、エントリ一覧、キーボード選択（`asdfjkl;`）
- **WM_PASTE によるペースト** — `AttachThreadInput` でフォーカス中のコントロールにペースト送信
- **重複排除** — SHA-256 コンテンツハッシュ、重複時は先頭に移動
- **SQLite 永続化** — DB 保存オプション、パス・上限（デフォルト 100 件）設定可能
- **設定画面** — ホットキー、永続化モード、DB パス、最大エントリ数、フォント、クリア操作
- **タスクトレイ** — 設定、再起動、終了
- **多言語対応** — 英語 / 日本語（外部 JSON ロケールファイル、`serde_json`）
- **Escape で閉じる** — Win32 `GetAsyncKeyState` ポーリング（Windows）

### 未実装

- 画像クリップボード監視 & サムネイル生成
- コピー元アプリ名の取得
- ピン留め機能
- Linux 対応
- アプリアイコン
- エラーハンドリング強化

## ビルド

```bash
cd better-clipboard
npm install
npm run build           # フロントエンド
cd src-tauri
cargo tauri dev         # 開発
cargo tauri build       # 本番
```

## 使い方

1. アプリはタスクトレイに常駐起動（ウィンドウなし）
2. `Alt+C` でクリップボードオーバーレイを表示
3. `a`-`;` キーでエントリを選択し、フォーカス中のアプリにペースト
4. `Esc` キーまたはクリックでオーバーレイを閉じる
5. トレイアイコンを右クリックで設定 / 再起動 / 終了

## 設定ファイル

`%LOCALAPPDATA%/BetterClipboard/config.toml` に保存。

```toml
[hotkeys]
overlay = "alt+c"
select_keys = "asdfjkl;"

persistence = "db"          # "session" | "db"

[db]
path = "..."                # デフォルト: %LOCALAPPDATA%/BetterClipboard/content.db

max_entries = 100
font_family = ""            # 空欄＝システム標準
locale = ""                 # ""＝自動検出, "en", "ja"
```
