use rusqlite::{Connection, params};
use std::path::Path;
use std::sync::Mutex;

pub struct Storage {
    conn: Mutex<Connection>,
}

impl Storage {
    pub fn new(db_path: &Path) -> Result<Self, String> {
        let conn = Connection::open(db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;

        let storage = Storage {
            conn: Mutex::new(conn),
        };
        storage.initialize_schema()?;
        Ok(storage)
    }

    fn initialize_schema(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute_batch("
            PRAGMA journal_mode=WAL;
            PRAGMA foreign_keys=ON;

            CREATE TABLE IF NOT EXISTS config (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS command_history (
                id          TEXT PRIMARY KEY,
                session_id  TEXT NOT NULL,
                command     TEXT NOT NULL,
                output      TEXT,
                exit_code   INTEGER,
                cwd         TEXT,
                shell_type  TEXT,
                started_at  TEXT NOT NULL,
                duration_ms INTEGER,
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_history_session ON command_history(session_id);
            CREATE INDEX IF NOT EXISTS idx_history_created ON command_history(created_at);

            CREATE TABLE IF NOT EXISTS ai_cache (
                prompt_hash   TEXT PRIMARY KEY,
                model         TEXT NOT NULL,
                response      TEXT NOT NULL,
                tokens_in     INTEGER,
                tokens_out    INTEGER,
                created_at    TEXT NOT NULL DEFAULT (datetime('now')),
                last_used_at  TEXT NOT NULL DEFAULT (datetime('now')),
                use_count     INTEGER NOT NULL DEFAULT 1
            );

            CREATE TABLE IF NOT EXISTS ai_conversations (
                id          TEXT PRIMARY KEY,
                title       TEXT,
                created_at  TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS ai_messages (
                id              TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL REFERENCES ai_conversations(id) ON DELETE CASCADE,
                role            TEXT NOT NULL,
                content         TEXT NOT NULL,
                model           TEXT,
                tokens_in       INTEGER,
                tokens_out      INTEGER,
                created_at      TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_messages_conversation ON ai_messages(conversation_id, created_at);

            CREATE TABLE IF NOT EXISTS schema_version (
                version    INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            INSERT OR IGNORE INTO schema_version (version) VALUES (1);
        ").map_err(|e| format!("Schema initialization failed: {}", e))?;

        Ok(())
    }

    pub fn get_config(&self, key: &str) -> Result<Option<String>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT value FROM config WHERE key = ?1")
            .map_err(|e| e.to_string())?;

        let result = stmt
            .query_row(params![key], |row| row.get::<_, String>(0))
            .ok();

        Ok(result)
    }

    pub fn set_config(&self, key: &str, value: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO config (key, value) VALUES (?1, ?2)",
            params![key, value],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_all_config(&self) -> Result<Vec<(String, String)>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT key, value FROM config")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| e.to_string())?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| e.to_string())?);
        }
        Ok(result)
    }

    pub fn save_command_history(
        &self,
        id: &str,
        session_id: &str,
        command: &str,
        output: Option<&str>,
        exit_code: Option<i32>,
        cwd: Option<&str>,
        shell_type: Option<&str>,
        started_at: &str,
        duration_ms: Option<i64>,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO command_history (id, session_id, command, output, exit_code, cwd, shell_type, started_at, duration_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![id, session_id, command, output, exit_code, cwd, shell_type, started_at, duration_ms],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}
