use std::fs;
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::gameloop::MyLittleUniverse;
use crate::station::StationState;
use crate::time::TimeStackState;

#[derive(Clone, PartialEq, Debug)]
pub enum ExternalSaveLoad {
    SaveTheUniverseAs(String),
    SaveTheUniverse
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

impl StationState {
    pub fn save(&self, universe_name: &String) {
        let universe_folder = save_file_path(&universe_name);
        let file_path = format!("{}{}", universe_folder, "stations/");
        create_dir_all(&file_path).expect("Had trouble creating stations save game folder.");
        let file_path = format!("{}{}.json", file_path, self.name());
        println!("Saving {}", file_path);
        let mut file = File::create(&file_path)
            .expect(&format!("Failed to create time save file, got: {}", &file_path).as_str());
        file.write_all(format!("{}", json!(self)).as_bytes()).expect("Had trouble saving station to file.");
    }
}

fn load_station(universe_name: &String, station_name: &String) -> StationState {
    let file_path = format!("{}{}{}.json", save_file_path(&universe_name), "stations/", &station_name);
    let mut file = File::open(&file_path)
        .expect(&format!("Filed to open station save file, got {}", &file_path));
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Failed to load time data");
    serde_json::from_str(&content).expect("Fauled to parse loaded station save file")
}

impl MyLittleUniverse {
    pub fn save(&self) -> ExternalSaveLoadReturnValue{
        self.time().save(&self.universe_name().to_string());
        self.station().save(&self.universe_name().to_string());
        ExternalSaveLoadReturnValue::UniverseIsSaved
    }
    pub fn save_as(&self, new_universe_name: &String) -> ExternalSaveLoadReturnValue{
        self.time().save(new_universe_name);
        self.station().save(new_universe_name);
        ExternalSaveLoadReturnValue::UniverseIsSaved
    }
}

pub fn load_universe(universe_name: String) -> MyLittleUniverse {
    let time = load_time(&universe_name);

    let mut stations = Vec::new();
    let station_dir = format!("./save/{}/stations/", universe_name);
    for station_file in fs::read_dir(&station_dir)
        .expect(format!("failed to list files in station, got: {}", &station_dir).as_str()) {
        match &station_file {
            Ok(file) => {
                let station_save_file = file.file_name().to_str().expect("Failed to get path to station save file").to_string();
                let station_save_file_without_type = station_save_file.split(".").next().expect(&format!("Failed getting filename for station, {:?}", station_save_file)).to_string();
                stations.push(load_station(&universe_name, &station_save_file_without_type));
            }
            Err(e) => { panic!("Were not able to load {:?}, got this error: {}", &station_file, e) }
        }
    };

    if stations.len() < 1 {
        println!("No stations were loaded, that is likely a mistake. Tried loadi")
    }

    MyLittleUniverse::new(universe_name.clone(), time, stations.get(0).unwrap().clone())
}

pub fn load_or_create_universe(universe_name: String) -> MyLittleUniverse {
    let save_file_path = format!("./save/{}/", universe_name);

    return if Path::new(&save_file_path).is_dir() {
        load_universe(universe_name)
    } else {
        MyLittleUniverse::new(universe_name.clone(), TimeStackState::new(), StationState::test_station())
    }
}

#[cfg(test)]
mod tests_int {
    use std::fs;
    use std::path::Path;

    use crate::gameloop::MyLittleUniverse;
    use crate::save_load::{load_or_create_universe, load_station, load_time, load_universe};
    use crate::station::StationState;
    use crate::time::TimeStackState;

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
    fn save_load_station() {
        let station = StationState::test_station();
        station.save(&"save_load_station".to_string());
        let loaded_state = load_station(&"save_load_station".to_string(), &station.name().to_string());
        assert_eq!(station, loaded_state);

        //Cleanup
        fs::remove_dir_all("./save/save_load_station/").expect("Had trouble cleanup after save_load_time");
    }

    #[test]
    fn save_load_universe() {
        let universe = MyLittleUniverse::new("save_load_universe".to_string(), TimeStackState::new(), StationState::test_station());
        universe.save();
        let loaded_universe = load_universe(universe.universe_name().to_string());
        assert_eq!(universe.universe_name(), loaded_universe.universe_name());
        assert_eq!(universe.time(), loaded_universe.time());
        assert_eq!(universe.station(), loaded_universe.station());

        //Cleanup
        fs::remove_dir_all("./save/save_load_universe/").expect("Had trouble cleanup after save_load_time");
    }

    #[test]
    fn load_or_create_universe_test() {
        assert_eq!(false, Path::new(&"./save/load_or_create_universe").is_dir());
        let universe = load_or_create_universe("load_or_create_universe".to_string());
        assert_eq!(false, Path::new(&"./save/load_or_create_universe").is_dir());
        universe.save();
        assert_eq!(true, Path::new(&"./save/load_or_create_universe").is_dir());

        //Cleanup
        fs::remove_dir_all("./save/load_or_create_universe/").expect("Had trouble cleanup after save_load_time");
    }
}