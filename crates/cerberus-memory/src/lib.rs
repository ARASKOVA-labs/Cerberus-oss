use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceRecord {
    pub id: Uuid,
    pub mission_id: Uuid,
    pub kind: String,
    pub summary: String,
    pub captured_at: OffsetDateTime,
}

pub struct StateDB {
    conn: Connection,
}

impl StateDB {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;

        // Use WAL mode for concurrency
        conn.pragma_update(None, "journal_mode", "WAL")?;

        Self::init_schema(&conn)?;

        Ok(Self { conn })
    }

    fn init_schema(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                source TEXT NOT NULL,
                started_at REAL NOT NULL,
                mission_data TEXT,
                plan_data TEXT
            );

            CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL REFERENCES sessions(id),
                role TEXT NOT NULL,
                content TEXT,
                timestamp REAL NOT NULL
            );

            CREATE TABLE IF NOT EXISTS evidence (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL REFERENCES sessions(id),
                kind TEXT NOT NULL,
                summary TEXT,
                captured_at REAL NOT NULL
            );

            CREATE TABLE IF NOT EXISTS findings (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL REFERENCES sessions(id),
                data TEXT NOT NULL
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
                content
            );

            CREATE TRIGGER IF NOT EXISTS messages_fts_insert AFTER INSERT ON messages BEGIN
                INSERT INTO messages_fts(rowid, content) VALUES (new.id, new.content);
            END;

            CREATE TRIGGER IF NOT EXISTS messages_fts_delete AFTER DELETE ON messages BEGIN
                DELETE FROM messages_fts WHERE rowid = old.id;
            END;

            CREATE TRIGGER IF NOT EXISTS messages_fts_update AFTER UPDATE ON messages BEGIN
                DELETE FROM messages_fts WHERE rowid = old.id;
                INSERT INTO messages_fts(rowid, content) VALUES (new.id, new.content);
            END;",
        )?;
        Ok(())
    }

    pub fn set_mission(&self, session_id: &str, mission_json: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE sessions SET mission_data = ?1 WHERE id = ?2",
            params![mission_json, session_id],
        )?;
        Ok(())
    }

    pub fn get_mission(&self, session_id: &str) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT mission_data FROM sessions WHERE id = ?1")?;
        let mut rows = stmt.query(params![session_id])?;
        if let Some(row) = rows.next()? {
            let data: Option<String> = row.get(0)?;
            return Ok(data);
        }
        Ok(None)
    }

    pub fn set_plan(&self, session_id: &str, plan_json: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE sessions SET plan_data = ?1 WHERE id = ?2",
            params![plan_json, session_id],
        )?;
        Ok(())
    }

    pub fn get_plan(&self, session_id: &str) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT plan_data FROM sessions WHERE id = ?1")?;
        let mut rows = stmt.query(params![session_id])?;
        if let Some(row) = rows.next()? {
            let data: Option<String> = row.get(0)?;
            return Ok(data);
        }
        Ok(None)
    }

    pub fn save_finding(
        &self,
        session_id: &str,
        finding_id: &str,
        finding_json: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO findings (id, session_id, data) VALUES (?1, ?2, ?3)",
            params![finding_id, session_id, finding_json],
        )?;
        Ok(())
    }

    pub fn get_findings(&self, session_id: &str) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT data FROM findings WHERE session_id = ?1")?;
        let rows = stmt.query_map(params![session_id], |row| row.get(0))?;
        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    pub fn create_session(&self, session_id: &str, source: &str) -> Result<()> {
        let now = OffsetDateTime::now_utc().unix_timestamp() as f64;
        self.conn.execute(
            "INSERT OR IGNORE INTO sessions (id, source, started_at) VALUES (?1, ?2, ?3)",
            params![session_id, source, now],
        )?;
        Ok(())
    }

    pub fn insert_message(&self, session_id: &str, role: &str, content: &str) -> Result<()> {
        let now = OffsetDateTime::now_utc().unix_timestamp() as f64;
        self.conn.execute(
            "INSERT INTO messages (session_id, role, content, timestamp) VALUES (?1, ?2, ?3, ?4)",
            params![session_id, role, content, now],
        )?;
        Ok(())
    }

    pub fn insert_evidence(&self, session_id: &str, record: &EvidenceRecord) -> Result<()> {
        let captured_at = record.captured_at.unix_timestamp() as f64;
        self.conn.execute(
            "INSERT INTO evidence (id, session_id, kind, summary, captured_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![record.id.to_string(), session_id, record.kind, record.summary, captured_at],
        )?;
        Ok(())
    }

    pub fn search_messages(&self, query: &str) -> Result<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT m.role, m.content 
             FROM messages_fts f 
             JOIN messages m ON f.rowid = m.id 
             WHERE messages_fts MATCH ?1 
             ORDER BY m.timestamp DESC",
        )?;
        let rows = stmt.query_map(params![query], |row| Ok((row.get(0)?, row.get(1)?)))?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    pub fn get_evidence(&self, session_id: &str) -> Result<Vec<EvidenceRecord>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, kind, summary, captured_at FROM evidence WHERE session_id = ?1")?;
        let rows = stmt.query_map(params![session_id], |row| {
            let id_str: String = row.get(0)?;
            let kind: String = row.get(1)?;
            let summary: String = row.get(2)?;
            let captured_at: f64 = row.get(3)?;
            Ok(EvidenceRecord {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                mission_id: Default::default(), // Backwards compat or drop field
                kind,
                summary,
                captured_at: OffsetDateTime::from_unix_timestamp(captured_at as i64)
                    .unwrap_or_else(|_| OffsetDateTime::now_utc()),
            })
        })?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }
}
