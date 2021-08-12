use std::{fs::File, io::BufReader, path::Path};

use chrono::NaiveDateTime;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct Update {
    pub date: NaiveDateTime,
    pub latest: String,
    pub updates: Vec<String>,
}

impl Update {
    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let u = serde_json::from_reader(reader)?;

        Ok(u)
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }
}
