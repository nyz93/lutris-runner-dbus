use crate::source::{lutris, steam};
use std::collections::HashMap;

use anyhow::Result;
use dbus::arg::{Append, Arg};
#[derive(Debug, Clone)]
pub struct Game {
    pub name: String,
    pub icon: String,
    pub category: String,
    pub directory: Option<String>,
    pub id: u32,
    pub src: String,
}
impl Arg for Game {
    const ARG_TYPE: dbus::arg::ArgType = dbus::arg::ArgType::Struct;

    fn signature() -> dbus::Signature<'static> {
        dbus::Signature::from("(sssida{sv})")
    }
}
impl Append for Game {
    fn append_by_ref(&self, iter: &mut dbus::arg::IterAppend) {
        use dbus::arg::{PropMap, RefArg, Variant};
        let mut props = PropMap::new();
        props.insert("category".to_string(), Variant(self.category.box_clone()));
        let mut actions = vec![];
        if self.directory.is_some() {
            actions.push("open-dir".to_owned());
        }
        props.insert("actions".to_string(), Variant(actions.box_clone()));
        iter.append((
            format!("{}_{}", self.src, self.id),
            &self.name,
            &self.icon,
            30,
            100.0,
            props,
        ))
    }
}

pub trait GameSource {
    fn query_game(&self, query: &String) -> Result<Vec<Game>>;
    fn run_game(&self, game: &Game) -> Result<()>;
    fn open_dir(&self, game: &Game) -> Result<()>;
}

type GameSourceBox = Box<dyn GameSource + Send>;
pub struct LocalLibrary {
    sources: HashMap<String, GameSourceBox>,
    matches: Vec<Game>,
}
pub trait Library {
    fn query_game(&mut self, query: &String) -> Result<Vec<Game>>;
    fn run_game(&self, id: String) -> Result<()>;
    fn open_dir(&self, id: String) -> Result<()>;
    fn teardown(&mut self);
}
impl LocalLibrary {
    pub fn new() -> Result<Self> {
        let mut sources: HashMap<String, GameSourceBox> = HashMap::new();
        sources.insert("lutris".to_string(), Box::new(lutris::Lutris::new()?));
        sources.insert("steam".to_string(), Box::new(steam::Steam::new()?));
        Ok(LocalLibrary {
            sources,
            matches: vec![],
        })
    }
    fn find_game(&self, id: String) -> Result<(&GameSourceBox, &Game)> {
        let game = self
            .matches
            .iter()
            .find(|g| format!("{}_{}", g.src, g.id) == id)
            .ok_or_else(|| anyhow::anyhow!("game match not found!"))?;
        Ok((&self.sources[&game.src], game))
    }
}
impl Library for LocalLibrary {
    fn query_game(&mut self, query: &String) -> Result<Vec<Game>> {
        self.matches = self
            .sources
            .values()
            .map(|src| src.query_game(&query))
            .flatten()
            .flatten()
            .collect();
        Ok(self.matches.iter().cloned().collect())
    }
    fn run_game(&self, id: String) -> Result<()> {
        let (src, game) = self.find_game(id)?;
        src.run_game(game)
    }
    fn open_dir(&self, id: String) -> Result<()> {
        let (src, game) = self.find_game(id)?;
        src.open_dir(game)
    }

    fn teardown(&mut self) {
        dbg!(&self.matches);
        self.matches.truncate(0);
    }
}
