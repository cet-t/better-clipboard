# Better Clipboard - カスタムクリップボードアプリ

## 概要

軽量で高速なクリップボードアプリ

## 技術スタック

- **フレームワーク**: Tauri (Rust + Web UI)
- **フロントエンド**: Web技術 (HTML/CSS/JS)
- **バックエンド**: Rust
- **DB**: SQLite

## 仕様

- タスクトレイに常駐
  - 右クリックメニューに以下の項目
    - 設定(%APPDATA%/BetterClipboard/config.toml)
      - ホットキーの設定
        - オーバーレイ表示キー(デフォルト: alt c)
        - 一覧からペースト項目選択キー(デフォルト: asdfjkl;)
      - 永続的に保存するか
        - セッションのみ
          - アプリ終了時にキャッシュを破棄
          - 次回起動時履歴は引き継がない
        - db 版
      - db 版選択時のみ
        - ファイルの配置場所(デフォルト: %APPDATA%/BetterClipboard/content.db)
      - 一覧のクリア
        - 表示のみ削除
        - DBも削除
        - 一定期間より古いものを削除
    - 再起動
      - db 版: キャッシュを保存
    - 終了
      - db 版: キャッシュを保存

- ホットキーを押すとオーバーレイが表示され、コピーしたテキスト、画像などのリストを表示する
  - 貼り付け: 選択後、フォーカス中のアプリケーションにペースト
  - テキスト編集: オーバーレイ上でエントリを選択後にテキストを編集可能（編集後は新規エントリとして保存）

- 履歴保存: テキスト → SQLite TEXT, 画像 → ファイルシステム(PNG) + DBにパス管理

- クリップボード監視
  - 抽象化レイヤーにより OS 固有実装を隠蔽
  - Windows: AddClipboardFormatListener
  - Linux: X11/Wayland 対応 (将来)

- ペースト方式
  - WM_PASTE をフォーカス中のコントロールに直接送信

- エントリ上限: デフォルト100件（設定で変更可能）
  - 上限超過時は `created_at` が最も古い非ピン留めエントリから削除

## DB スキーマ

```sql
CREATE TABLE clipboard_entries (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_type      TEXT NOT NULL CHECK(entry_type IN ('text', 'image')),
    content_hash    TEXT NOT NULL,                       -- SHA256（重複排除用）
    text_content    TEXT,                                -- entry_type='text' 時のみ
    file_path       TEXT,                                -- entry_type='image' 時: 保存先パス
    thumbnail_path  TEXT,                                -- オーバーレイ表示用サムネイル
    file_size       INTEGER,                             -- バイト数
    source_app      TEXT,                                -- コピー元アプリ名（参考）
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    is_pinned       INTEGER NOT NULL DEFAULT 0,
    display_order   INTEGER NOT NULL                     -- 表示順（asdfjkl;選択キーに対応）
);

CREATE INDEX idx_clipboard_entries_hash ON clipboard_entries(content_hash);
CREATE INDEX idx_clipboard_entries_order ON clipboard_entries(display_order);
CREATE INDEX idx_clipboard_entries_created ON clipboard_entries(created_at DESC);
```

- 重複検出: `content_hash` が既存エントリと同一の場合、新規追加せず `created_at` を更新（最前面に移動）
- 画像ファイルの保存先: `<content.db のディレクトリ>/images/<id>.png`

## 対象プラットフォーム

- Windows 11
- Linux (optional)
