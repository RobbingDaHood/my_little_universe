use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::construct::construct::Construct;
use crate::MainConfig;
use crate::my_little_universe::MyLittleUniverse;
use crate::time::TimeStackState;
use crate::universe_generator::generate_universe;

#[derive(Clone, PartialEq, Debug)]
pub enum ExternalSaveLoad {
    SaveTheUniverseAs(String),
    SaveTheUniverse,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ExternalSaveLoadReturnValue {
    UniverseIsSaved
}

impl TimeStackState {
    pub fn save(&self, universe_name: &String) {
        let file_path = format!("{}time.json", save_file_path(&universe_name));
        let mut file = File::create(file_path)
            .expect("Failed to create time save file");
        file.write_all(format!("{}", json!(self)).as_bytes())
            .expect("Had problem saving time");
    }
}

fn load_time(universe_name: &String) -> TimeStackState {
    let file_path = format!("{}time.json", save_file_path(&universe_name));
    let mut file = File::open(&file_path)
        .expect(&format!("Filed to open time save file: {}", &file_path));
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Failed to load time data");
    serde_json::from_str(&content).expect("Fauled to parse loaded time save file")
}

fn save_file_path(universe_name: &String) -> String {
    let path = format!("./save/{}/", universe_name);
    create_dir_all(&path).expect("Hard trouble creating save game folder.");
    path
}

impl MyLittleUniverse {
    pub fn save(&self) -> ExternalSaveLoadReturnValue {
        self.time().save(&self.universe_name().to_string());
        Self::save_constructs(self, &self.universe_name().to_string());
        ExternalSaveLoadReturnValue::UniverseIsSaved
    }

    pub fn save_as(&self, new_universe_name: &String) -> ExternalSaveLoadReturnValue {
        self.time().save(new_universe_name);
        Self::save_constructs(self, new_universe_name);
        ExternalSaveLoadReturnValue::UniverseIsSaved
    }

    fn save_constructs(&self, universe_name: &String) {
        let universe_folder = save_file_path(&universe_name);
        let file_path = format!("{}{}", universe_folder, "constructs.json");
        println!("Saving {}", file_path);
        let mut file = File::create(&file_path)
            .expect(&format!("Failed to create time save file, got: {}", &file_path).as_str());
        file.write_all(format!("{}", json!(self.constructs())).as_bytes()).expect("Had trouble saving station to file.");
    }
}

pub fn load_universe(universe_name: String) -> MyLittleUniverse {
    let time = load_time(&universe_name);
    let constructs = load_constructs(&universe_name);
    MyLittleUniverse::new(universe_name.clone(), time, constructs)
}

fn load_constructs(universe_name: &String) -> HashMap<String, Construct> {
    let file_path = format!("./save/{}/constructs.json", universe_name);
    let mut file = File::open(&file_path)
        .expect(&format!("Filed to open constructs save file, got {}", &file_path));
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Failed to load constructs data");
    let constructs: HashMap<String, Construct> = serde_json::from_str(&content).expect("Failed to parse loaded constructs save file");

    if constructs.len() < 1 {
        println!("No constructs were loaded, that is likely a mistake.")
    }
    constructs
}

pub fn load_or_create_universe(config: &MainConfig) -> MyLittleUniverse {
    let save_file_path = format!("./save/{}/", config.universe_name());

    return if Path::new(&save_file_path).is_dir() {
        load_universe(config.universe_name().to_string())
    } else {
        generate_universe(config)
    };
}

#[cfg(test)]
mod tests_int {
    use std::fs;
    use std::path::Path;
    use crate::MainConfig;

    use crate::save_load::{load_or_create_universe, load_time, load_universe};
    use crate::time::TimeStackState;
    use crate::universe_generator::generate_simple_universe;

    #[test]
    fn save_load_time() {
        let time_state = TimeStackState::new();
        time_state.save(&"save_load_time".to_string());
        let loaded_state = load_time(&"save_load_time".to_string());
        assert_eq!(time_state, loaded_state);

        //Cleanup
        fs::remove_dir_all("./save/save_load_time/").expect("Had trouble cleanup after save_load_time");
    }

    #[test]
    fn save_load_universe() {
        let universe = generate_simple_universe("save_load_universe".to_string());
        universe.save();
        let loaded_universe = load_universe(universe.universe_name().to_string());
        assert_eq!(universe.universe_name(), loaded_universe.universe_name());
        assert_eq!(universe.time(), loaded_universe.time());
        assert_eq!(universe.constructs(), loaded_universe.constructs());

        //Cleanup
        fs::remove_dir_all("./save/save_load_universe/").expect("Had trouble cleanup after save_load_time");
    }

    #[test]
    fn load_or_create_universe_test() {
        let main_config = MainConfig {
            address : "random".to_string(),
            universe_name : "load_or_create_universe".to_string(),
            config_name : "default".to_string()
        };

        assert_eq!(false, Path::new(&"./save/load_or_create_universe").is_dir());
        let universe = load_or_create_universe(&main_config);
        assert_eq!(false, Path::new(&"./save/load_or_create_universe").is_dir());
        universe.save();
        assert_eq!(true, Path::new(&"./save/load_or_create_universe").is_dir());

        //Cleanup
        fs::remove_dir_all("./save/load_or_create_universe/").expect("Had trouble cleanup after save_load_time");
    }
}