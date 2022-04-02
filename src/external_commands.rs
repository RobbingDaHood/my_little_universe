use serde::{Deserialize, Serialize};
use crate::construct::construct;

use crate::construct::construct::{ConstructEvenReturnType, ExternalConstructEventType};
use crate::my_little_universe::MyLittleUniverseReturnValues;
use crate::products::Product;
use crate::save_load::{ExternalSaveLoad, ExternalSaveLoadReturnValue};
use crate::station::{ExternalStationEventType, LoadingRequest, StationEvenReturnType};
use crate::time::{ExternalTimeEventType, TimeEventReturnType};

#[derive(Clone, PartialEq, Debug)]
pub enum ExternalCommands {
    Time(ExternalTimeEventType),
    Station(String, ExternalStationEventType),
    Save(ExternalSaveLoad),
    Construct(String, ExternalConstructEventType),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ExternalCommandReturnValues {
    Time(TimeEventReturnType),
    Station(StationEvenReturnType),
    Save(ExternalSaveLoadReturnValue),
    Universe(MyLittleUniverseReturnValues),
    Construct(ConstructEvenReturnType),
}

impl TryFrom<&String> for ExternalCommands {
    type Error = String;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        let command_parts = value.split(" ").collect::<Vec<&str>>();

        if command_parts.len() == 0 {
            return Err("Command is empty".to_string());
        }

        return match command_parts[0] {
            "Time" => { Self::parse_time(command_parts) }
            "Station" => { Self::parse_station(command_parts) }
            "Construct" => { Self::parse_construct(command_parts) }
            "Save" => { Self::parse_save_load(command_parts) }
            _ => { Err(format!("Unknown command, got: {}", value)) }
        };
    }
}

impl ExternalCommands {
    fn parse_time(command_parts: Vec<&str>) -> Result<Self, String> {
        if command_parts.len() < 2 {
            return Err(format!("Time command needs at least the command name. Got {:?}", command_parts));
        }

        match command_parts[1] {
            "Pause" => { return Ok(ExternalCommands::Time(ExternalTimeEventType::Pause)); }
            "Start" => { return Ok(ExternalCommands::Time(ExternalTimeEventType::Start)); }
            "StartUntilTurn" => {
                if command_parts.len() > 2 {
                    if let Ok(turn_number) = command_parts[2].parse::<u64>() {
                        return Ok(ExternalCommands::Time(ExternalTimeEventType::StartUntilTurn(turn_number)));
                    }
                }
                return Err(format!("StartUntilTurn need a u64 integer argument. Got {:?}", command_parts));
            }
            "SetSpeed" => {
                if command_parts.len() > 2 {
                    if let Ok(turn_min_duration_in_milli_secs) = command_parts[2].parse::<u64>() {
                        return Ok(ExternalCommands::Time(ExternalTimeEventType::SetSpeed(turn_min_duration_in_milli_secs)));
                    }
                }
                return Err(format!("SetSpeed need a u64 integer argument. Got {:?}", command_parts));
            }
            "GetTimeStackState" => {
                if command_parts.len() > 2 {
                    if let Ok(include_stack) = command_parts[2].parse::<bool>() {
                        return Ok(ExternalCommands::Time(ExternalTimeEventType::GetTimeStackState { include_stack }));
                    }
                } else {
                    return Ok(ExternalCommands::Time(ExternalTimeEventType::GetTimeStackState { include_stack: true }));
                }
                return Err(format!("GetTimeStackState optinal booĺ include_stack. Got {:?}", command_parts));
            }
            _ => Err(format!("Unknown Time command. Got {:?}", command_parts))
        }
    }

    fn parse_station(command_parts: Vec<&str>) -> Result<Self, String> {
        if command_parts.len() < 3 {
            return Err(format!("Station command needs at least the station name and command name. Got {:?}", command_parts));
        }

        let station_name = command_parts[1];

        match command_parts[2] {
            "RequestLoad" => {
                if command_parts.len() > 4 {
                    let product = match command_parts[3] {
                        "Ores" => { Some(Product::Ores) }
                        "Metals" => { Some(Product::Metals) }
                        "PowerCells" => { Some(Product::PowerCells) }
                        _ => { None }
                    };

                    if let Some(product_value) = product {
                        if let Ok(amount) = command_parts[4].parse::<u32>() {
                            return Ok(ExternalCommands::Station(station_name.to_string(), ExternalStationEventType::RequestLoad(LoadingRequest::new(product_value, amount))));
                        }
                    }
                }
                return Err(format!("RequestLoad need Product and u32 amount. Got {:?}", command_parts));
            }
            "RequestUnload" => {
                if command_parts.len() > 4 {
                    let product = match command_parts[3] {
                        "Ores" => { Some(Product::Ores) }
                        "Metals" => { Some(Product::Metals) }
                        "PowerCells" => { Some(Product::PowerCells) }
                        _ => { None }
                    };

                    if let Some(product_value) = product {
                        if let Ok(amount) = command_parts[4].parse::<u32>() {
                            return Ok(ExternalCommands::Station(station_name.to_string(), ExternalStationEventType::RequestUnload(LoadingRequest::new(product_value, amount))));
                        }
                    }
                }
                return Err(format!("RequestUnload need Product and u32 amount. Got {:?}", command_parts));
            }
            "GetStationState" => {
                if command_parts.len() > 3 {
                    if let Ok(include_stack) = command_parts[3].parse::<bool>() {
                        return Ok(ExternalCommands::Station(station_name.to_string(), ExternalStationEventType::GetStationState { include_stack }));
                    }
                } else {
                    return Ok(ExternalCommands::Station(station_name.to_string(), ExternalStationEventType::GetStationState { include_stack: true }));
                }
                return Err(format!("GetStationState optinal booĺ include_stack. Got {:?}", command_parts));
            }
            _ => Err(format!("Unknown Station command. Got {:?}", command_parts))
        }
    }

    fn parse_construct(command_parts: Vec<&str>) -> Result<Self, String> {
        if command_parts.len() < 3 {
            return Err(format!("Construct command needs at least the Construct name and command name. Got {:?}", command_parts));
        }

        let construct_name = command_parts[1];

        match command_parts[2] {
            "RequestLoad" => {
                if command_parts.len() > 4 {
                    let product = match command_parts[3] {
                        "Ores" => { Some(Product::Ores) }
                        "Metals" => { Some(Product::Metals) }
                        "PowerCells" => { Some(Product::PowerCells) }
                        _ => { None }
                    };

                    if let Some(product_value) = product {
                        if let Ok(amount) = command_parts[4].parse::<u32>() {
                            return Ok(ExternalCommands::Construct(construct_name.to_string(), ExternalConstructEventType::RequestLoad(construct::LoadingRequest::new(product_value, amount))));
                        }
                    }
                }
                return Err(format!("RequestLoad need Product and u32 amount. Got {:?}", command_parts));
            }
            "RequestUnload" => {
                if command_parts.len() > 4 {
                    let product = match command_parts[3] {
                        "Ores" => { Some(Product::Ores) }
                        "Metals" => { Some(Product::Metals) }
                        "PowerCells" => { Some(Product::PowerCells) }
                        _ => { None }
                    };

                    if let Some(product_value) = product {
                        if let Ok(amount) = command_parts[4].parse::<u32>() {
                            return Ok(ExternalCommands::Construct(construct_name.to_string(), ExternalConstructEventType::RequestUnload(construct::LoadingRequest::new(product_value, amount))));
                        }
                    }
                }
                return Err(format!("RequestUnload need Product and u32 amount. Got {:?}", command_parts));
            }
            "ConstructState" => {
                if command_parts.len() > 3 {
                    if let Ok(include_stack) = command_parts[3].parse::<bool>() {
                        return Ok(ExternalCommands::Construct(construct_name.to_string(), ExternalConstructEventType::GetConstructState { include_stack }));
                    }
                } else {
                    return Ok(ExternalCommands::Construct(construct_name.to_string(), ExternalConstructEventType::GetConstructState { include_stack: true }));
                }
                return Err(format!("GetConstructState optinal booĺ include_stack. Got {:?}", command_parts));
            }
            _ => Err(format!("Unknown Construct command. Got {:?}", command_parts))
        }
    }

    fn parse_save_load(command_parts: Vec<&str>) -> Result<ExternalCommands, String> {
        if command_parts.len() < 2 {
            return Err(format!("Save command needs at least the command name. Got {:?}", command_parts));
        }

        match command_parts[1] {
            "SaveTheUniverse" => { return Ok(ExternalCommands::Save(ExternalSaveLoad::SaveTheUniverse)); }
            "SaveTheUniverseAs" => {
                if command_parts.len() < 3 {
                    return Err(format!("SaveTheUniverseAs needs a name of the save folder. Got {:?}", command_parts));
                }

                return Ok(ExternalCommands::Save(ExternalSaveLoad::SaveTheUniverseAs(command_parts[2].to_string())));
            }
            _ => Err(format!("Unknown Save command. Got {:?}", command_parts))
        }
    }
}

#[cfg(test)]
mod tests_int {
    use crate::external_commands::ExternalCommands;
    use crate::products::Product;
    use crate::save_load::ExternalSaveLoad;
    use crate::station::{ExternalStationEventType, LoadingRequest};
    use crate::time::ExternalTimeEventType;

    #[test]
    fn it_works() {
        assert_eq!(ExternalCommands::Time(ExternalTimeEventType::Pause),
                   ExternalCommands::try_from(&"Time Pause".to_string()).unwrap());
        assert_eq!(ExternalCommands::Time(ExternalTimeEventType::Start),
                   ExternalCommands::try_from(&"Time Start".to_string()).unwrap());
        assert_eq!(ExternalCommands::Time(ExternalTimeEventType::StartUntilTurn(22)),
                   ExternalCommands::try_from(&"Time StartUntilTurn 22".to_string()).unwrap());
        assert_eq!(ExternalCommands::Time(ExternalTimeEventType::SetSpeed(23)),
                   ExternalCommands::try_from(&"Time SetSpeed 23".to_string()).unwrap());
        assert_eq!(ExternalCommands::Time(ExternalTimeEventType::GetTimeStackState { include_stack: true }),
                   ExternalCommands::try_from(&"Time GetTimeStackState".to_string()).unwrap());


        assert_eq!(ExternalCommands::Station("name".to_string(), ExternalStationEventType::RequestLoad(LoadingRequest::new(Product::PowerCells, 24))),
                   ExternalCommands::try_from(&"Station name RequestLoad PowerCells 24".to_string()).unwrap());
        assert_eq!(ExternalCommands::Station("name".to_string(), ExternalStationEventType::RequestUnload(LoadingRequest::new(Product::Ores, 25))),
                   ExternalCommands::try_from(&"Station name RequestUnload Ores 25".to_string()).unwrap());
        assert_eq!(ExternalCommands::Station("name".to_string(), ExternalStationEventType::GetStationState { include_stack: true }),
                   ExternalCommands::try_from(&"Station name GetStationState".to_string()).unwrap());

        assert_eq!(ExternalCommands::Save(ExternalSaveLoad::SaveTheUniverse),
                   ExternalCommands::try_from(&"Save SaveTheUniverse".to_string()).unwrap());
        assert_eq!(ExternalCommands::Save(ExternalSaveLoad::SaveTheUniverseAs("new_name".to_string())),
                   ExternalCommands::try_from(&"Save SaveTheUniverseAs new_name".to_string()).unwrap());
    }
}
