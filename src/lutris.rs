use anyhow::Result;
use rusqlite::{params, Connection};
use std::path::PathBuf;

#[derive(Debug)]
pub struct Game {
    pub name: String,
    pub slug: String,
    pub runner: String,
    pub directory: Option<String>,
    pub id: u32,
}

pub struct Lutris {
    sql: Connection,
}

impl Lutris {
    pub fn new() -> Result<Self> {
        let base = xdg::BaseDirectories::with_prefix("lutris")?;
        let db = PathBuf::from(base.find_data_file("pga.db").unwrap());
        let sql = Connection::open_with_flags(db, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        Ok(Lutris { sql })
    }

    pub fn query_game(&self, query: String) -> Result<Vec<Game>> {
        let mut cmd = self
            .sql
            .prepare("SELECT id, name, slug, runner, directory FROM games WHERE name LIKE '%' || ? || '%' AND installed = 1")?;
        let iter = cmd.query_and_then(params![query], |row| -> Result<Game> {
            Ok(Game {
                id: row.get(0)?,
                name: row.get(1)?,
                slug: row.get(2)?,
                runner: row.get(3)?,
                directory: row.get::<_, Option<String>>(4)?.and_then(|c| {
                    if c.as_str() == "" {
                        None
                    } else {
                        Some(c)
                    }
                }),
            })
        })?;
        iter.collect()
    }

    pub fn run_game(&self, id: String) -> Result<()> {
        use std::process::{Command, Stdio};
        Ok(Command::new("lutris")
            .arg(format!("lutris:rungameid/{}", id))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map(drop)?)
    }

    pub fn open_dir(&self, id: String) -> Result<()> {
        let mut cmd = self
            .sql
            .prepare("SELECT directory FROM games WHERE id = ? LIMIT 1")?;
        let mut iter = cmd.query_and_then(params![id], |row| -> Result<Option<String>> {
            Ok(row.get(0)?)
        })?;
        if let Some(Ok(Some(dir))) = iter.next() {
            open::that(dir)?;
        }
        Ok(())
    }
}
