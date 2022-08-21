use crate::library::{Game, GameSource};
use anyhow::Result;
pub struct Steam {}

impl Steam {
    pub fn new() -> Result<Self> {
        Ok(Steam {})
    }
}

impl GameSource for Steam {
    fn query_game(&self, query: &String) -> Result<Vec<Game>> {
        Ok(vec![])
    }

    fn run_game(&self, game: &Game) -> Result<()> {
        Ok(())
    }

    fn open_dir(&self, game: &Game) -> Result<()> {
        Ok(())
    }
}
#[cfg(test)]
mod test {
    use super::Steam;
    use crate::library::GameSource;

    #[test]
    fn steam() {}
}
