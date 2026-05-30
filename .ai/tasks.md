# Better Clipboard - タスク管理

## 凡例

- [x] 完了
- [ ] 未完了
- [~] 進行中

---

## Phase 1: プロジェクト基盤 ✅

- [x] spec 策定（plan.md）
- [x] Tauri プロジェクトスキャフォールド
- [x] 依存関係設定
- [x] モジュール構成作成（config, db, clipboard, tray）
- [x] オーバーレイウィンドウ設定
- [x] Tauri コマンド登録
- [x] フロントエンド仮配置
- [x] ビルド確認

## Phase 2: コア機能 ✅

- [x] グローバルホットキー（alt c → オーバーレイ表示）
- [x] ペースト機構（WM_PASTE + AttachThreadInput）
- [x] クリップボード監視 → DB保存
- [x] 重複排除（content_hash）
- [x] セルフトリガー抑制（AtomicBool）
- [x] オーバーレイ表示時にエントリ再読込

## Phase 3: UI ✅

- [x] オーバーレイUI デザイン
  - [x] ダークテーマ、エントリ一覧、キーヒント、ホバー効果
  - [x] キーボード選択（asdfjkl;）
  - [x] Escape で閉じる
  - [x] クリックでペースト
- [x] 設定画面UI
  - [x] settings.html / settings.ts / settings.css
  - [x] ホットキー設定・永続化モード・DBパス・エントリ上限
  - [x] クリア操作（表示のみ / すべて / 期間指定）
- [x] トレイ「設定」→ 設定画面表示
- [x] Vite multi-page build（index + settings）

## Phase 4: 仕上げ

- [ ] テキスト編集編集機能（オーバーレイ上でエントリのテキストを編集、編集後は新規エントリとして保存）
- [ ] 画像のサムネイル生成
- [ ] 画像クリップボード監視
- [ ] ソースアプリ名取得
- [ ] ピン留め機能
- [ ] Linux 対応
- [ ] アイコン作成
- [ ] エラーハンドリング強化
