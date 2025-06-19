use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};

#[derive(Debug, Serialize, Deserialize)]
pub struct PermanentUpgrades {
    pub echoes: i32,

    pub bonus_hp: i32,
    pub bonus_power: i32,
    pub bonus_defense: i32,
}

impl PermanentUpgrades {
    pub fn new() -> Self {
        PermanentUpgrades {
            echoes: 0,
            bonus_hp: 0,
            bonus_power: 0,
            bonus_defense: 0,
        }
    }
}

pub fn save_meta(upgrades: &PermanentUpgrades) -> Result<(), Box<dyn Error>> {
    let save_data = serde_json::to_string(upgrades)?;
    let mut file = File::create("meta")?;
    file.write_all(save_data.as_bytes())?;
    Ok(())
}

pub fn load_meta() -> Result<PermanentUpgrades, Box<dyn Error>> {
    let mut json_save_state = String::new();
    let mut file = File::open("meta")?;
    file.read_to_string(&mut json_save_state)?;
    let result = serde_json::from_str::<PermanentUpgrades>(&json_save_state)?;
    Ok(result)
}
