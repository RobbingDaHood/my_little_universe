use crate::products::Product;
use crate::station::{ExternalStationEventType, LoadingRequest, StationEvenReturnType};
use crate::time::{ExternalTimeEventType, TimeEventReturnType};

#[derive(Clone, PartialEq, Debug)]
pub enum ExternalCommands {
    Time(ExternalTimeEventType),
    Station(ExternalStationEventType),
}

#[derive(Clone, PartialEq, Debug)]
pub enum ExternalCommandReturnValues {
    Time(TimeEventReturnType),
    Station(StationEvenReturnType),
}

impl TryFrom<String> for ExternalCommands {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let command_parts = value.split(" ").collect::<Vec<&str>>();

        if command_parts.len() == 0 {
            return Err("Command is empty".to_string());
        }

        return match command_parts[0] {
            "Time" => { Self::parse_time(command_parts) }
            "Station" => { Self::parse_station(command_parts) }
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
                    if let Ok(turn_number) = command_parts[2].parse::<u128>() {
                        return Ok(ExternalCommands::Time(ExternalTimeEventType::StartUntilTurn(turn_number)));
                    }
                }
                return Err(format!("StartUntilTurn need a u128 integer argument. Got {:?}", command_parts));
            }
            "SetSpeed" => {
                if command_parts.len() > 2 {
                    if let Ok(turn_min_duration_in_milli_secs) = command_parts[2].parse::<u64>() {
                        return Ok(ExternalCommands::Time(ExternalTimeEventType::SetSpeed(turn_min_duration_in_milli_secs)));
                    }
                }
                return Err(format!("SetSpeed need a u64 integer argument. Got {:?}", command_parts));
            }
            "GetTimeStackState" => { return Ok(ExternalCommands::Time(ExternalTimeEventType::GetTimeStackState)); }
            _ => Err(format!("Unknown Time command. Got {:?}", command_parts))
        }
    }

    fn parse_station(command_parts: Vec<&str>) -> Result<Self, String> {
        if command_parts.len() < 2 {
            return Err(format!("Station command needs at least the command name. Got {:?}", command_parts));
        }
        match command_parts[1] {
            "RequestLoad" => {
                if command_parts.len() > 3 {
                    let product = match command_parts[2] {
                        "Ores" => { Some(Product::Ores) }
                        "Metals" => { Some(Product::Metals) }
                        "PowerCells" => { Some(Product::PowerCells) }
                        _ => { None }
                    };

                    if let Some(product_value) = product {
                        if let Ok(amount) = command_parts[3].parse::<u32>() {
                            return Ok(ExternalCommands::Station(ExternalStationEventType::RequestLoad(LoadingRequest::new(product_value, amount))));
                        }
                    }
                }
                return Err(format!("RequestLoad need Product and u32 amount. Got {:?}", command_parts));
            }
            "RequestUnload" => {
                if command_parts.len() > 3 {
                    let product = match command_parts[2] {
                        "Ores" => { Some(Product::Ores) }
                        "Metals" => { Some(Product::Metals) }
                        "PowerCells" => { Some(Product::PowerCells) }
                        _ => { None }
                    };

                    if let Some(product_value) = product {
                        if let Ok(amount) = command_parts[3].parse::<u32>() {
                            return Ok(ExternalCommands::Station(ExternalStationEventType::RequestUnload(LoadingRequest::new(product_value, amount))));
                        }
                    }
                }
                return Err(format!("RequestUnload need Product and u32 amount. Got {:?}", command_parts));
            }
            "GetStationState" => { return Ok(ExternalCommands::Station(ExternalStationEventType::GetStationState)); }
            _ => Err(format!("Unknown Station command. Got {:?}", command_parts))
        }
    }
}

#[cfg(test)]
mod tests_int {
    use crate::external_commands::ExternalCommands;
    use crate::products::Product;
    use crate::station::{ExternalStationEventType, LoadingRequest};
    use crate::time::ExternalTimeEventType;

    #[test]
    fn it_works() {
        assert_eq!(ExternalCommands::Time(ExternalTimeEventType::Pause),
                   ExternalCommands::try_from("Time Pause".to_string()).unwrap());
        assert_eq!(ExternalCommands::Time(ExternalTimeEventType::Start),
                   ExternalCommands::try_from("Time Start".to_string()).unwrap());
        assert_eq!(ExternalCommands::Time(ExternalTimeEventType::StartUntilTurn(22)),
                   ExternalCommands::try_from("Time StartUntilTurn 22".to_string()).unwrap());
        assert_eq!(ExternalCommands::Time(ExternalTimeEventType::SetSpeed(23)),
                   ExternalCommands::try_from("Time SetSpeed 23".to_string()).unwrap());
        assert_eq!(ExternalCommands::Time(ExternalTimeEventType::GetTimeStackState),
                   ExternalCommands::try_from("Time GetTimeStackState".to_string()).unwrap());


        assert_eq!(ExternalCommands::Station(ExternalStationEventType::RequestLoad(LoadingRequest::new(Product::PowerCells, 24))),
                   ExternalCommands::try_from("Station RequestLoad PowerCells 24".to_string()).unwrap());
        assert_eq!(ExternalCommands::Station(ExternalStationEventType::RequestUnload(LoadingRequest::new(Product::Ores, 25))),
                   ExternalCommands::try_from("Station RequestUnload Ores 25".to_string()).unwrap());
        assert_eq!(ExternalCommands::Station(ExternalStationEventType::GetStationState),
                   ExternalCommands::try_from("Station GetStationState".to_string()).unwrap());
    }
}
