use rusqlite::{Connection, Result, params};
use std::path::Path;

pub struct Db {
    conn: Connection,
}

pub struct PasswordEntry {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub created_at: String,
}

/// DBファイルを所有者のみ読み書きできるようにする（新規作成時も既存ファイルも 600）
#[cfg(unix)]
fn restrict_permissions(path: &Path) -> std::io::Result<()> {
    use std::fs::{OpenOptions, Permissions};
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

    OpenOptions::new().write(true).create(true).mode(0o600).open(path)?;
    std::fs::set_permissions(path, Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

impl Db {
    pub fn open(path: &Path) -> Result<Self> {
        if let Err(e) = restrict_permissions(path) {
            eprintln!("Warning: failed to restrict permissions of {} ({}).", path.display(), e);
        }
        let conn = Connection::open(path)?;
        // 削除・更新した値（旧パスワード等）がファイルの空き領域に残らないようにする
        conn.execute_batch("PRAGMA secure_delete = ON;")?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS password_history (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                service    TEXT NOT NULL,
                username   TEXT NOT NULL DEFAULT '',
                password   TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_service
                ON password_history(service);
            CREATE TABLE IF NOT EXISTS service_tags (
                service TEXT NOT NULL,
                tag     TEXT NOT NULL,
                PRIMARY KEY (service, tag)
            );",
        )?;

        // 既存DBにusernameカラムがなければ追加
        let has_username: bool = conn
            .prepare("PRAGMA table_info(password_history)")?
            .query_map([], |row| row.get::<_, String>(1))?
            .any(|name| name.unwrap_or_default() == "username");
        if !has_username {
            conn.execute_batch(
                "ALTER TABLE password_history ADD COLUMN username TEXT NOT NULL DEFAULT '';",
            )?;
        }

        Ok(Self { conn })
    }

    pub fn save(&self, service: &str, username: &str, password: &str) -> Result<()> {
        let now = chrono::Local::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO password_history (service, username, password, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![service, username, password, now],
        )?;
        Ok(())
    }

    /// サービスの履歴から最新の空でないユーザ名を取得
    pub fn latest_username(&self, service: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT username
             FROM password_history
             WHERE service = ?1 AND username <> ''
             ORDER BY created_at DESC, id DESC
             LIMIT 1",
        )?;
        let mut rows = stmt.query(params![service])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }

    pub fn get_history(&self, service: &str) -> Result<Vec<PasswordEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, username, password, created_at
             FROM password_history
             WHERE service = ?1
             ORDER BY created_at DESC",
        )?;
        let entries = stmt
            .query_map(params![service], |row| {
                Ok(PasswordEntry {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    password: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(entries)
    }

    pub fn list_services(&self) -> Result<Vec<(String, usize)>> {
        let mut stmt = self.conn.prepare(
            "SELECT service, COUNT(*) as cnt
             FROM password_history
             GROUP BY service
             ORDER BY service",
        )?;
        let rows = stmt
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, usize>(1)?)))?
            .collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }

    /// 指定IDのユーザ名・パスワードを更新（None の項目は変更しない）
    pub fn update_entry(
        &self,
        id: i64,
        username: Option<&str>,
        password: Option<&str>,
    ) -> Result<usize> {
        let n = self.conn.execute(
            "UPDATE password_history
             SET username = COALESCE(?2, username),
                 password = COALESCE(?3, password)
             WHERE id = ?1",
            params![id, username, password],
        )?;
        Ok(n)
    }

    /// 全レコードの (ID, 保存されているパスワード) を取得
    pub fn all_passwords(&self) -> Result<Vec<(i64, String)>> {
        let mut stmt = self.conn.prepare("SELECT id, password FROM password_history ORDER BY id")?;
        let rows = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }

    /// 複数レコードのパスワードを1トランザクションで書き換え、VACUUM で旧データを消す
    pub fn replace_passwords(&mut self, rows: &[(i64, String)]) -> Result<()> {
        let tx = self.conn.transaction()?;
        for (id, password) in rows {
            tx.execute(
                "UPDATE password_history SET password = ?2 WHERE id = ?1",
                params![id, password],
            )?;
        }
        tx.commit()?;
        self.conn.execute_batch("VACUUM;")?;
        Ok(())
    }

    /// サービスの履歴件数
    pub fn count_service(&self, service: &str) -> Result<usize> {
        self.conn.query_row(
            "SELECT COUNT(*) FROM password_history WHERE service = ?1",
            params![service],
            |row| row.get(0),
        )
    }

    /// サービス名を変更（変更した件数を返す）。タグも新しいサービス名に引き継ぐ（統合時は和集合）
    pub fn rename_service(&self, old: &str, new: &str) -> Result<usize> {
        let tx = self.conn.unchecked_transaction()?;
        let n = tx.execute(
            "UPDATE password_history SET service = ?2 WHERE service = ?1",
            params![old, new],
        )?;
        if n > 0 {
            tx.execute(
                "INSERT OR IGNORE INTO service_tags (service, tag)
                 SELECT ?2, tag FROM service_tags WHERE service = ?1",
                params![old, new],
            )?;
            tx.execute("DELETE FROM service_tags WHERE service = ?1", params![old])?;
        }
        tx.commit()?;
        Ok(n)
    }

    /// 指定IDの (サービス名, ユーザ名, 保存されているパスワード) を取得（該当なしは None）
    pub fn get_entry(&self, id: i64) -> Result<Option<(String, String, String)>> {
        match self.conn.query_row(
            "SELECT service, username, password FROM password_history WHERE id = ?1",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ) {
            Ok(entry) => Ok(Some(entry)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// 指定IDのレコードを削除（削除したレコードのサービス名を返す。該当なしは None）
    pub fn delete_entry(&self, id: i64) -> Result<Option<String>> {
        let service: Option<String> = match self.conn.query_row(
            "SELECT service FROM password_history WHERE id = ?1",
            params![id],
            |row| row.get(0),
        ) {
            Ok(s) => Some(s),
            Err(rusqlite::Error::QueryReturnedNoRows) => None,
            Err(e) => return Err(e),
        };
        if service.is_some() {
            self.conn.execute("DELETE FROM password_history WHERE id = ?1", params![id])?;
        }
        Ok(service)
    }

    /// サービスの全履歴を削除（タグも削除する）
    pub fn delete_service(&self, service: &str) -> Result<usize> {
        let tx = self.conn.unchecked_transaction()?;
        let n = tx.execute(
            "DELETE FROM password_history WHERE service = ?1",
            params![service],
        )?;
        tx.execute("DELETE FROM service_tags WHERE service = ?1", params![service])?;
        tx.commit()?;
        Ok(n)
    }

    /// サービスにタグを付ける（付いていなかったタグの数を返す）
    pub fn add_tags(&self, service: &str, tags: &[String]) -> Result<usize> {
        let tx = self.conn.unchecked_transaction()?;
        let mut n = 0;
        for tag in tags {
            n += tx.execute(
                "INSERT OR IGNORE INTO service_tags (service, tag) VALUES (?1, ?2)",
                params![service, tag],
            )?;
        }
        tx.commit()?;
        Ok(n)
    }

    /// サービスからタグを外す（外したタグの数を返す）
    pub fn remove_tags(&self, service: &str, tags: &[String]) -> Result<usize> {
        let tx = self.conn.unchecked_transaction()?;
        let mut n = 0;
        for tag in tags {
            n += tx.execute(
                "DELETE FROM service_tags WHERE service = ?1 AND tag = ?2",
                params![service, tag],
            )?;
        }
        tx.commit()?;
        Ok(n)
    }

    /// サービスに付いているタグ（タグ名順）
    pub fn tags_of(&self, service: &str) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT tag FROM service_tags WHERE service = ?1 ORDER BY tag")?;
        let tags = stmt
            .query_map(params![service], |row| row.get(0))?
            .collect::<Result<Vec<_>>>()?;
        Ok(tags)
    }

    /// 履歴のあるサービスに付いている (サービス名, タグ) の一覧（サービス名・タグ名順）
    pub fn all_service_tags(&self) -> Result<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT service, tag FROM service_tags
             WHERE service IN (SELECT DISTINCT service FROM password_history)
             ORDER BY service, tag",
        )?;
        let rows = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }
}
