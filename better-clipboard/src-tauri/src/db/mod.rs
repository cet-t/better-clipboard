use rusqlite::{params, Connection, Result as SqlResult};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct ClipboardEntry {
    pub id: i64,
    pub entry_type: String,
    pub content_hash: String,
    pub text_content: Option<String>,
    pub file_path: Option<String>,
    pub thumbnail_path: Option<String>,
    pub file_size: Option<i64>,
    pub source_app: Option<String>,
    pub created_at: String,
    pub is_pinned: bool,
    pub display_order: i64,
}

pub struct Database {
    conn: Connection,
    max_entries: usize,
}

impl Database {
    pub fn open(path: &Path, max_entries: usize) -> SqlResult<Self> {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(path)?;
        let db = Self { conn, max_entries };
        db.initialize()?;
        Ok(db)
    }

    fn initialize(&self) -> SqlResult<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS clipboard_entries (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                entry_type      TEXT NOT NULL CHECK(entry_type IN ('text', 'image')),
                content_hash    TEXT NOT NULL,
                text_content    TEXT,
                file_path       TEXT,
                thumbnail_path  TEXT,
                file_size       INTEGER,
                source_app      TEXT,
                created_at      TEXT NOT NULL DEFAULT (datetime('now')),
                is_pinned       INTEGER NOT NULL DEFAULT 0,
                display_order   INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_clipboard_entries_hash
                ON clipboard_entries(content_hash);
            CREATE INDEX IF NOT EXISTS idx_clipboard_entries_order
                ON clipboard_entries(display_order);
            CREATE INDEX IF NOT EXISTS idx_clipboard_entries_created
                ON clipboard_entries(created_at DESC);",
        )?;
        Ok(())
    }

    pub fn insert_or_update(&self, entry: &ClipboardEntry) -> SqlResult<()> {
        let existing: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM clipboard_entries WHERE content_hash = ?1 AND entry_type = ?2",
                params![entry.content_hash, entry.entry_type],
                |row| row.get(0),
            )
            .ok();

        if let Some(existing_id) = existing {
            self.conn.execute(
                "UPDATE clipboard_entries SET created_at = datetime('now'), display_order = ?1 WHERE id = ?2",
                params![entry.display_order, existing_id],
            )?;
        } else {
            self.conn.execute(
                "INSERT INTO clipboard_entries (entry_type, content_hash, text_content, file_path, thumbnail_path, file_size, source_app, is_pinned, display_order)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    entry.entry_type,
                    entry.content_hash,
                    entry.text_content,
                    entry.file_path,
                    entry.thumbnail_path,
                    entry.file_size,
                    entry.source_app,
                    entry.is_pinned,
                    entry.display_order,
                ],
            )?;
        }

        self.enforce_limit()?;
        Ok(())
    }

    fn enforce_limit(&self) -> SqlResult<()> {
        self.conn.execute(
            "DELETE FROM clipboard_entries
             WHERE id NOT IN (
                 SELECT id FROM clipboard_entries
                 WHERE is_pinned = 1
                 ORDER BY created_at DESC
                 LIMIT ?1
             ) AND is_pinned = 0 AND id NOT IN (
                 SELECT id FROM clipboard_entries
                 ORDER BY created_at DESC
                 LIMIT ?1
             )",
            params![self.max_entries as i64],
        )?;
        Ok(())
    }

    pub fn get_id_by_hash(&self, content_hash: &str, entry_type: &str) -> SqlResult<Option<i64>> {
        let result = self.conn.query_row(
            "SELECT id FROM clipboard_entries WHERE content_hash = ?1 AND entry_type = ?2",
            params![content_hash, entry_type],
            |row| row.get(0),
        );
        Ok(result.ok())
    }

    pub fn next_display_order(&self) -> SqlResult<i64> {
        let max: i64 = self.conn.query_row(
            "SELECT COALESCE(MAX(display_order), 0) FROM clipboard_entries",
            [],
            |row| row.get(0),
        )?;
        Ok(max + 1)
    }

    pub fn get_recent(&self, limit: usize) -> SqlResult<Vec<ClipboardEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, entry_type, content_hash, text_content, file_path,
                    thumbnail_path, file_size, source_app, created_at, is_pinned, display_order
             FROM clipboard_entries
             ORDER BY display_order DESC, created_at DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(ClipboardEntry {
                id: row.get(0)?,
                entry_type: row.get(1)?,
                content_hash: row.get(2)?,
                text_content: row.get(3)?,
                file_path: row.get(4)?,
                thumbnail_path: row.get(5)?,
                file_size: row.get(6)?,
                source_app: row.get(7)?,
                created_at: row.get(8)?,
                is_pinned: row.get::<_, i32>(9)? != 0,
                display_order: row.get(10)?,
            })
        })?;
        rows.collect()
    }

    pub fn count_entries(&self) -> SqlResult<i64> {
        self.conn
            .query_row("SELECT COUNT(*) FROM clipboard_entries", [], |row| {
                row.get(0)
            })
    }

    pub fn delete_entry(&self, id: i64) -> SqlResult<()> {
        self.conn
            .execute("DELETE FROM clipboard_entries WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn update_entry_text(&self, id: i64, text: &str, content_hash: &str) -> SqlResult<()> {
        self.conn.execute(
            "UPDATE clipboard_entries SET text_content = ?1, content_hash = ?2 WHERE id = ?3",
            params![text, content_hash, id],
        )?;
        Ok(())
    }

    pub fn clear_all(&self) -> SqlResult<()> {
        self.conn.execute("DELETE FROM clipboard_entries", [])?;
        Ok(())
    }

    pub fn clear_display_only(&self) -> SqlResult<()> {
        let max_order = self.max_entries as i64;
        self.conn.execute(
            "DELETE FROM clipboard_entries WHERE display_order > ?1",
            params![max_order],
        )?;
        Ok(())
    }

    pub fn clear_older_than(&self, days: i64) -> SqlResult<()> {
        self.conn.execute(
            "DELETE FROM clipboard_entries
             WHERE is_pinned = 0
             AND datetime(created_at) < datetime('now', '-?1 days')",
            params![days],
        )?;
        Ok(())
    }
}
