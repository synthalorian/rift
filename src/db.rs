use chrono::Utc;
use rusqlite::{params, Connection};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

/// Tracks asset state: hash, last conversion, errors
pub struct AssetDb {
    conn: Mutex<Connection>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssetRecord {
    pub relative_path: String,
    pub sha256: String,
    pub last_modified: String,
    pub last_converted: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PipelineRun {
    pub id: String,
    pub timestamp: String,
    pub status: String,
    pub total_assets: u32,
    pub converted: u32,
    pub errors: u32,
    pub summary: Option<String>,
}

impl AssetDb {
    pub fn open(path: &Path) -> crate::Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS assets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                relative_path TEXT NOT NULL UNIQUE,
                sha256 TEXT NOT NULL,
                last_modified TEXT NOT NULL,
                last_converted TEXT,
                status TEXT NOT NULL DEFAULT 'pending',
                error_message TEXT
            );
            CREATE TABLE IF NOT EXISTS pipeline_runs (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'running',
                total_assets INTEGER DEFAULT 0,
                converted INTEGER DEFAULT 0,
                errors INTEGER DEFAULT 0,
                summary TEXT
            );
            CREATE TABLE IF NOT EXISTS asset_errors (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                run_id TEXT,
                relative_path TEXT NOT NULL,
                error_type TEXT NOT NULL,
                message TEXT NOT NULL,
                timestamp TEXT NOT NULL
            );",
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn upsert_asset(&self, path: &str, sha256: &str, last_modified: &str) -> crate::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO assets (relative_path, sha256, last_modified, status)
             VALUES (?1, ?2, ?3, 'pending')
             ON CONFLICT(relative_path) DO UPDATE SET
                sha256 = excluded.sha256,
                last_modified = excluded.last_modified,
                status = CASE WHEN assets.sha256 != excluded.sha256 THEN 'pending' ELSE assets.status END",
            params![path, sha256, last_modified],
        )?;
        Ok(())
    }

    pub fn mark_converted(&self, path: &str) -> crate::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE assets SET status = 'ok', last_converted = ?1 WHERE relative_path = ?2",
            params![Utc::now().to_rfc3339(), path],
        )?;
        Ok(())
    }

    pub fn mark_error(&self, path: &str, message: &str) -> crate::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE assets SET status = 'error', error_message = ?1 WHERE relative_path = ?2",
            params![message, path],
        )?;
        Ok(())
    }

    pub fn log_error(
        &self,
        run_id: &str,
        path: &str,
        error_type: &str,
        message: &str,
    ) -> crate::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO asset_errors (run_id, relative_path, error_type, message, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![run_id, path, error_type, message, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn needs_conversion(&self, path: &str) -> crate::Result<bool> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT status FROM assets WHERE relative_path = ?1")?;
        let result: std::result::Result<String, _> =
            stmt.query_row(params![path], |row| row.get(0));
        match result {
            Ok(status) => Ok(status == "pending" || status == "error"),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(true),
            Err(e) => Err(e.into()),
        }
    }

    pub fn create_run(&self, id: &str) -> crate::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO pipeline_runs (id, timestamp, status) VALUES (?1, ?2, 'running')",
            params![id, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn complete_run(&self, id: &str, converted: u32, errors: u32) -> crate::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE pipeline_runs SET status = 'completed', converted = ?1, errors = ?2 WHERE id = ?3",
            params![converted, errors, id],
        )?;
        Ok(())
    }

    pub fn get_asset_counts(&self) -> crate::Result<HashMap<String, u32>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT status, COUNT(*) as count FROM assets GROUP BY status")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
        })?;
        let mut counts = HashMap::new();
        for row in rows {
            let (status, count) = row?;
            counts.insert(status, count);
        }
        Ok(counts)
    }

    pub fn get_recent_runs(&self, limit: u32) -> crate::Result<Vec<PipelineRun>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, status, total_assets, converted, errors, summary
             FROM pipeline_runs ORDER BY timestamp DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], |row| {
            Ok(PipelineRun {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                status: row.get(2)?,
                total_assets: row.get::<_, u32>(3).unwrap_or(0),
                converted: row.get::<_, u32>(4).unwrap_or(0),
                errors: row.get::<_, u32>(5).unwrap_or(0),
                summary: row.get(6)?,
            })
        })?;
        let mut runs = Vec::new();
        for row in rows {
            runs.push(row?);
        }
        Ok(runs)
    }

    pub fn get_asset_errors(&self, limit: u32) -> crate::Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT ae.run_id, ae.relative_path, ae.error_type, ae.message, ae.timestamp
             FROM asset_errors ae
             ORDER BY ae.timestamp DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], |row| {
            Ok(serde_json::json!({
                "run_id": row.get::<_, String>(0)?,
                "relative_path": row.get::<_, String>(1)?,
                "error_type": row.get::<_, String>(2)?,
                "message": row.get::<_, String>(3)?,
                "timestamp": row.get::<_, String>(4)?,
            }))
        })?;
        let mut errors = Vec::new();
        for row in rows {
            errors.push(row?);
        }
        Ok(errors)
    }

    pub fn get_assets(
        &self,
        status_filter: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> crate::Result<Vec<AssetRecord>> {
        let conn = self.conn.lock().unwrap();
        let (sql, params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = match status_filter {
            Some(s) => (
                "SELECT relative_path, sha256, last_modified, last_converted, status, error_message
                 FROM assets WHERE status = ?1 ORDER BY last_modified DESC LIMIT ?2 OFFSET ?3"
                    .into(),
                vec![
                    Box::new(s.to_string()),
                    Box::new(limit as i64),
                    Box::new(offset as i64),
                ],
            ),
            None => (
                "SELECT relative_path, sha256, last_modified, last_converted, status, error_message
                 FROM assets ORDER BY last_modified DESC LIMIT ?1 OFFSET ?2"
                    .into(),
                vec![Box::new(limit as i64), Box::new(offset as i64)],
            ),
        };
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(
            rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
            |row| {
                Ok(AssetRecord {
                    relative_path: row.get(0)?,
                    sha256: row.get(1)?,
                    last_modified: row.get(2)?,
                    last_converted: row.get(3)?,
                    status: row.get(4)?,
                    error_message: row.get(5)?,
                })
            },
        )?;
        let mut assets = Vec::new();
        for row in rows {
            assets.push(row?);
        }
        Ok(assets)
    }

    pub fn hash_file(path: &Path) -> crate::Result<String> {
        let mut hasher = Sha256::new();
        let mut file = std::fs::File::open(path)?;
        std::io::copy(&mut file, &mut hasher)?;
        Ok(hex::encode(hasher.finalize()))
    }

    pub fn close(self) -> crate::Result<()> {
        let conn = self.conn.into_inner().unwrap();
        conn.close()
            .map_err(|(_, e)| crate::RiftError::Database(e))?;
        Ok(())
    }
}
