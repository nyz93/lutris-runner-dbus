use crate::library::{Game, GameSource};
use anyhow::Result;
use rusqlite::{params, Connection};
use std::path::PathBuf;

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
}

impl GameSource for Lutris {
    fn query_game(&self, query: &String) -> Result<Vec<Game>> {
        let mut cmd = self
            .sql
            .prepare("SELECT id, name, slug, runner, directory FROM games WHERE name LIKE '%' || ? || '%' AND installed = 1 AND runner != 'steam';")?;
        let iter = cmd.query_and_then(params![query], |row| -> Result<Game> {
            Ok(Game {
                src: "lutris".to_string(),
                id: row.get(0)?,
                name: row.get(1)?,
                icon: format!("lutris_{}", row.get::<_, String>(2)?),
                category: format!("Lutris ({})", row.get::<_, String>(3)?),
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

    fn run_game(&self, g: &Game) -> Result<()> {
        use std::process::{Command, Stdio};
        Ok(Command::new("lutris")
            .arg(format!("lutris:rungameid/{}", g.id))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map(drop)?)
    }

    fn open_dir(&self, g: &Game) -> Result<()> {
        let mut cmd = self
            .sql
            .prepare("SELECT directory FROM games WHERE id = ? LIMIT 1")?;
        let mut iter = cmd.query_and_then(params![g.id], |row| -> Result<Option<String>> {
            Ok(row.get(0)?)
        })?;
        if let Some(Ok(Some(dir))) = iter.next() {
            open::that(dir)?;
        }
        Ok(())
    }
}
