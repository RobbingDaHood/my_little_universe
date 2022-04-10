use serde::{Deserialize, Serialize};

pub use crate::construct::amount::Amount;
use crate::construct::construct::{ConstructEvenReturnType, ExternalConstructEventType};
use crate::construct::construct_position::ExternalConstructPositionEventType;
use crate::my_little_universe::{ExternalUniverseEventType, MyLittleUniverseReturnValues, OfMove, OfTransferCargo};
use crate::products::Product;
use crate::save_load::{ExternalSaveLoad, ExternalSaveLoadReturnValue};
use crate::sector::{ExternalSectorEventType, SectorEvenReturnType, SectorPosition};
use crate::time::{ExternalTimeEventType, TimeEventReturnType};

#[derive(Clone, PartialEq, Debug)]
pub enum ExternalCommands {
    Time(ExternalTimeEventType),
    Save(ExternalSaveLoad),
    Construct(String, ExternalConstructEventType),
    Sector(SectorPosition, ExternalSectorEventType),
    Universe(ExternalUniverseEventType),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ExternalCommandReturnValues {
    Time(TimeEventReturnType),
    Save(ExternalSaveLoadReturnValue),
    Universe(MyLittleUniverseReturnValues),
    Construct(ConstructEvenReturnType),
    Sector(SectorEvenReturnType),
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
            "Construct" => { Self::parse_construct(command_parts) }
            "Sector" => { Self::parse_sector(command_parts) }
            "Save" => { Self::parse_save_load(command_parts) }
            _ => { Self::parse_universe(command_parts) }
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

    fn parse_construct(command_parts: Vec<&str>) -> Result<Self, String> {
        if command_parts.len() < 3 {
            return Err(format!("Construct command needs at least the Construct name and command name. Got {:?}", command_parts));
        }

        let construct_name = command_parts[1];

        match command_parts[2] {
            "GetConstructState" => {
                if command_parts.len() > 3 {
                    if let Ok(include_stack) = command_parts[3].parse::<bool>() {
                        return Ok(ExternalCommands::Construct(construct_name.to_string(), ExternalConstructEventType::GetConstructState { include_stack }));
                    }
                } else {
                    return Ok(ExternalCommands::Construct(construct_name.to_string(), ExternalConstructEventType::GetConstructState { include_stack: true }));
                }
                return Err(format!("GetConstructState optional booĺ include_stack. Got {:?}", command_parts));
            }
            _ => Err(format!("Unknown Construct command. Got {:?}", command_parts))
        }
    }

    fn parse_sector(command_parts: Vec<&str>) -> Result<Self, String> {
        if command_parts.len() < 3 {
            return Err(format!("Construct command needs at least the Construct name and command name. Got {:?}", command_parts));
        }

        let sector_position = command_parts[1];
        let sector_coordinates = sector_position.split("-").collect::<Vec<&str>>();
        let sector_position = SectorPosition::new(
            sector_coordinates[0].parse::<u8>().expect(format!("Had trouble parsing {} to u8", sector_coordinates[0]).as_str()),
            sector_coordinates[1].parse::<u8>().expect(format!("Had trouble parsing {} to u8", sector_coordinates[1]).as_str()),
            sector_coordinates[2].parse::<u8>().expect(format!("Had trouble parsing {} to u8", sector_coordinates[2]).as_str()),
        );

        match command_parts[2] {
            "GetSectorState" => {
                return Ok(ExternalCommands::Sector(sector_position, ExternalSectorEventType::GetSectorState));
            }
            "MoveToGroup" => {
                if command_parts.len() > 4 {
                    if let Ok(group_address) = command_parts[4].parse::<usize>() {
                        return Ok(ExternalCommands::Sector(sector_position, ExternalSectorEventType::MoveToGroup(command_parts[3].to_string(), Some(group_address))));
                    }
                } else if command_parts.len() > 3 {
                    return Ok(ExternalCommands::Sector(sector_position, ExternalSectorEventType::MoveToGroup(command_parts[3].to_string(), None)));
                }
                return Err(format!("MoveToGroup could not parse command. Got {:?}", command_parts));
            }
            _ => Err(format!("Unknown Sector command. Got {:?}", command_parts))
        }
    }

    fn parse_save_load(command_parts: Vec<&str>) -> Result<ExternalCommands, String> {
        if command_parts.len() < 2 {
            return Err(format!("Save command needs at least the command name. Got {:?}", command_parts));
        }

        match command_parts[1] {
            "TheUniverse" => { return Ok(ExternalCommands::Save(ExternalSaveLoad::TheUniverse)); }
            "TheUniverseAs" => {
                if command_parts.len() < 3 {
                    return Err(format!("SaveTheUniverseAs needs a name of the save folder. Got {:?}", command_parts));
                }

                return Ok(ExternalCommands::Save(ExternalSaveLoad::TheUniverseAs(command_parts[2].to_string())));
            }
            _ => Err(format!("Unknown Save command. Got {:?}", command_parts))
        }
    }

    fn parse_universe(command_parts: Vec<&str>) -> Result<ExternalCommands, String> {
        if command_parts.len() < 1 {
            return Err(format!("Need at least the command name. Got Nothing: {:?}", command_parts));
        }

        match command_parts[0] {
            "Move" => {
                if command_parts.len() > 2 {
                    // One is construct name

                    let sector_position = command_parts[2];
                    let sector_position = sector_position.split("-").collect::<Vec<&str>>();
                    let sector_position = SectorPosition::new(
                        sector_position[0].parse::<u8>().expect(format!("Had trouble parsing {} to u8", sector_position[0]).as_str()),
                        sector_position[1].parse::<u8>().expect(format!("Had trouble parsing {} to u8", sector_position[1]).as_str()),
                        sector_position[2].parse::<u8>().expect(format!("Had trouble parsing {} to u8", sector_position[2]).as_str()),
                    );

                    if command_parts.len() > 3 {
                        if let Ok(group_address) = command_parts[3].parse::<usize>() {
                            return Ok(ExternalCommands::Universe(ExternalUniverseEventType::Move(
                                OfMove::new(
                                    command_parts[1].to_string(),
                                    sector_position,
                                    Some(group_address),
                                )
                            )));
                        }
                    } else {
                        return Ok(ExternalCommands::Universe(ExternalUniverseEventType::Move(
                            OfMove::new(
                                command_parts[1].to_string(),
                                sector_position,
                                None,
                            )
                        )));
                    }
                }
                return Err(format!("MoveToSector did not parse the command. Got {:?}", command_parts));
            }
            "TransferCargo" => {
                if command_parts.len() > 4 {
                    let product = match command_parts[3] {
                        "Ores" => { Some(Product::Ores) }
                        "Metals" => { Some(Product::Metals) }
                        "PowerCells" => { Some(Product::PowerCells) }
                        _ => { None }
                    };

                    if let Some(product_value) = product {
                        if let Ok(amount) = command_parts[4].parse::<u32>() {
                            return Ok(ExternalCommands::Universe(ExternalUniverseEventType::TransferCargo(OfTransferCargo::new(command_parts[1].to_string(), command_parts[2].to_string(), Amount::new(product_value, amount)))));
                        }
                    }
                }
                return Err(format!("TransferCargo need source_construct_name target_construct_name product amount. Got {:?}", command_parts));
            }
            "Dock" => {
                if command_parts.len() > 2 {
                    return Ok(ExternalCommands::Construct(command_parts[1].to_string(), ExternalConstructEventType::ConstructPosition(ExternalConstructPositionEventType::Dock(command_parts[2].to_string()))));
                }
                return Err(format!("Dock need source_construct_name target_construct_name. Got {:?}", command_parts));
            }
            "Undock" => {
                if command_parts.len() > 1 {
                    return Ok(ExternalCommands::Construct(command_parts[1].to_string(), ExternalConstructEventType::ConstructPosition(ExternalConstructPositionEventType::Undock)));
                }
                return Err(format!("Undock need source_construct_name. Got {:?}", command_parts));
            }
            _ => Err(format!("Unknown Universe command. Got {:?}", command_parts))
        }
    }
}

#[cfg(test)]
mod tests_int {
    use crate::construct::construct::ExternalConstructEventType;
    use crate::construct::construct_position::ExternalConstructPositionEventType;
    use crate::external_commands::{Amount, ExternalCommands};
    use crate::my_little_universe::{ExternalUniverseEventType, OfMove, OfTransferCargo};
    use crate::products::Product;
    use crate::save_load::ExternalSaveLoad;
    use crate::sector::{ExternalSectorEventType, SectorPosition};
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

        assert_eq!(ExternalCommands::Construct("name".to_string(), ExternalConstructEventType::GetConstructState { include_stack: true }),
                   ExternalCommands::try_from(&"Construct name GetConstructState".to_string()).unwrap());

        assert_eq!(ExternalCommands::Sector(SectorPosition::new(1, 1, 1), ExternalSectorEventType::GetSectorState),
                   ExternalCommands::try_from(&"Sector 1-1-1 GetSectorState".to_string()).unwrap());

        assert_eq!(
            ExternalCommands::Universe(ExternalUniverseEventType::Move(
                OfMove::new("the_construct".to_string(), SectorPosition::new(1, 1, 1), None)
            )),
            ExternalCommands::try_from(&"Move the_construct 1-1-1".to_string()).unwrap()
        );
        assert_eq!(
            ExternalCommands::Universe(ExternalUniverseEventType::Move(
                OfMove::new("the_construct".to_string(), SectorPosition::new(1, 1, 1), Some(22))
            )),
            ExternalCommands::try_from(&"Move the_construct 1-1-1 22".to_string()).unwrap()
        );
        assert_eq!(
            ExternalCommands::Universe(ExternalUniverseEventType::TransferCargo(
                OfTransferCargo::new("the_construct_1".to_string(), "the_construct_2".to_string(), Amount::new(Product::Ores, 25))
            )),
            ExternalCommands::try_from(&"TransferCargo the_construct_1 the_construct_2 Ores 25".to_string()).unwrap()
        );
        assert_eq!(
            ExternalCommands::Construct(
                "the_construct_1".to_string(),
                ExternalConstructEventType::ConstructPosition(
                    ExternalConstructPositionEventType::Dock("the_construct_2".to_string())
                ),
            ),
            ExternalCommands::try_from(&"Dock the_construct_1 the_construct_2".to_string()).unwrap()
        );
        assert_eq!(
            ExternalCommands::Construct(
                "the_construct_1".to_string(),
                ExternalConstructEventType::ConstructPosition(ExternalConstructPositionEventType::Undock),
            ),
            ExternalCommands::try_from(&"Undock the_construct_1".to_string()).unwrap()
        );

        assert_eq!(ExternalCommands::Save(ExternalSaveLoad::TheUniverse),
                   ExternalCommands::try_from(&"Save TheUniverse".to_string()).unwrap());
        assert_eq!(ExternalCommands::Save(ExternalSaveLoad::TheUniverseAs("new_name".to_string())),
                   ExternalCommands::try_from(&"Save TheUniverseAs new_name".to_string()).unwrap());
    }
}
