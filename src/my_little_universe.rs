use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{ExternalCommandReturnValues, ExternalCommands};
use crate::construct::construct::{Construct, ConstructEvenReturnType, ConstructEventType, ExternalConstructEventType, InternalConstructEventType};
use crate::construct::construct_position::{ConstructPositionEventReturnType, ConstructPositionEventType, ConstructPositionSector, ConstructPositionStatus, ExternalConstructPositionEventType, InternalConstructPositionEventType};
use crate::external_commands::Amount;
use crate::save_load::ExternalSaveLoad;
use crate::sector::{ExternalSectorEventType, InternalSectorEventType, Sector, SectorEvenReturnType, SectorEventType, SectorPosition};
use crate::sector::SectorEvenReturnType::{Denied, Entered};
use crate::time::{InternalTimeEventType, TimeEventType, TimeStackState};

pub struct MyLittleUniverse {
    time: TimeStackState,
    pub(crate) constructs: HashMap<String, Construct>,
    sectors: HashMap<SectorPosition, Sector>,
    universe_name: String,
}


#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ExternalUniverseEventType {
    Move(OfMove),
    TransferCargo(OfTransferCargo),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct OfTransferCargo {
    source_construct_name: String,
    target_construct_name: String,
    amount: Amount,
}

impl OfTransferCargo {
    pub fn new(source_construct_name: String, target_construct_name: String, amount: Amount) -> Self {
        OfTransferCargo { source_construct_name, target_construct_name, amount }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct OfMove {
    construct_name: String,
    sector_position: SectorPosition,
    group_address: Option<usize>,
}

impl OfMove {
    pub fn new(construct_name: String, sector_position: SectorPosition, group_address: Option<usize>) -> Self {
        OfMove { construct_name, sector_position, group_address }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum MyLittleUniverseReturnValues {
    CouldNotFindStation,
    CouldNotFindConstruct(String),
    CouldNotFindSector(SectorPosition),
    MovedToSector(usize),
    CouldNotMoveToSector(String),
    Denied(String),
    CargoTransfered(u32),
}

impl MyLittleUniverse {
    pub fn new(universe_name: String, time: TimeStackState, constructs: HashMap<String, Construct>, sectors: HashMap<SectorPosition, Sector>) -> Self {
        MyLittleUniverse {
            time,
            universe_name,
            constructs,
            sectors,
        }
    }
    pub fn time(&self) -> &TimeStackState {
        &self.time
    }
    pub fn constructs(&self) -> &HashMap<String, Construct> {
        &self.constructs
    }
    pub fn universe_name(&self) -> &str {
        &self.universe_name
    }

    pub fn handle_event(&mut self, event: ExternalCommands) -> ExternalCommandReturnValues {
        match event {
            ExternalCommands::Time(time_event) => {
                let return_type = self.time.push_event(&TimeEventType::External(time_event));
                ExternalCommandReturnValues::Time(return_type)
            }
            ExternalCommands::Construct(construct_name, construct_event) => {
                match construct_event {
                    ExternalConstructEventType::ConstructPosition(ExternalConstructPositionEventType::Undock) => {
                        let docked_at_name = match self.constructs.get(&construct_name) {
                            Some(construct) => {
                                match construct.position.position() {
                                    ConstructPositionStatus::Sector(sector_name) =>
                                        return ExternalCommandReturnValues::Construct(ConstructEvenReturnType::ConstructPosition(ConstructPositionEventReturnType::Denied(format!("Cannot undock because is not docked. Is in sector {:?}", sector_name)))),
                                    ConstructPositionStatus::Docked(the_docked_at_name) => the_docked_at_name.clone()
                                }
                            }
                            None => { return ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::CouldNotFindConstruct(construct_name)); }
                        };

                        self.constructs.get_mut(&docked_at_name)
                            .expect("The construct does not exists in universe list of constructs")
                            .position.handle_event(&ConstructPositionEventType::Internal(InternalConstructPositionEventType::Undocked(construct_name.clone())));

                        let sector_position = self.get_sector_position(construct_name.clone()).clone();
                        let construct = self.constructs.get_mut(&construct_name).unwrap();
                        let return_type = construct.push_event(&ConstructEventType::Internal(InternalConstructEventType::ConstructPosition(InternalConstructPositionEventType::Undock(sector_position.clone()))));

                        ExternalCommandReturnValues::Construct(return_type)
                    }
                    ExternalConstructEventType::ConstructPosition(ExternalConstructPositionEventType::Dock(target_construct_name)) => {
                        return ExternalCommandReturnValues::Construct(ConstructEvenReturnType::ConstructPosition(self.handle_docking_request(construct_name, target_construct_name)));
                    }
                    _ => {
                        return match self.constructs.get_mut(&construct_name) {
                            Some(construct) => {
                                let return_type = construct.push_event(&ConstructEventType::External(construct_event));
                                ExternalCommandReturnValues::Construct(return_type)
                            }
                            None => { ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::CouldNotFindConstruct(construct_name)) }
                        };
                    }
                }
            }
            ExternalCommands::Save(save_event) => {
                match save_event {
                    ExternalSaveLoad::TheUniverseAs(universe_name) => {
                        ExternalCommandReturnValues::Save(self.save_as(&universe_name))
                    }
                    ExternalSaveLoad::TheUniverse => {
                        ExternalCommandReturnValues::Save(self.save())
                    }
                }
            }
            ExternalCommands::Sector(sector_position, sector_event) => {
                return match self.sectors.get_mut(&sector_position) {
                    Some(sector) => {
                        match sector_event {
                            ExternalSectorEventType::MoveToGroup(ref construct_name, _) => {
                                let return_type = sector.push_event(&SectorEventType::External(sector_event.clone()));

                                if let SectorEvenReturnType::Entered(group_address) = return_type {
                                    return match self.constructs.get_mut(construct_name.as_str()).expect("Could not find construct!!")
                                        .push_event(&ConstructEventType::External(ExternalConstructEventType::ConstructPosition(ExternalConstructPositionEventType::EnterGroup(group_address)))) {
                                        ConstructEvenReturnType::ConstructPosition(ConstructPositionEventReturnType::RequestProcessed) => {
                                            ExternalCommandReturnValues::Sector(return_type)
                                        }
                                        event => ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::CouldNotMoveToSector(format!("Construct {} had trouble updating its group state to {}, got this event {:?}", construct_name, group_address, event)))
                                    };
                                }

                                ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::CouldNotMoveToSector(format!("Construct {} had trouble sector {:?}, got this returned {:?}", construct_name, sector_position, return_type)))
                            }
                            _ => {
                                let return_type = sector.push_event(&SectorEventType::External(sector_event));
                                ExternalCommandReturnValues::Sector(return_type)
                            }
                        }
                    }
                    None => { ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::CouldNotFindSector(sector_position)) }
                };
            }
            ExternalCommands::Universe(event) => {
                match event {
                    ExternalUniverseEventType::Move(of_move_to_sector) => ExternalCommandReturnValues::Universe(self.move_to_sector(of_move_to_sector)),
                    ExternalUniverseEventType::TransferCargo(transfer_cargo) => {
                        if transfer_cargo.source_construct_name.eq(&transfer_cargo.target_construct_name) {
                            return ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::Denied(format!("Cannot transfer because source {} and target {} construct is the same", transfer_cargo.source_construct_name, transfer_cargo.target_construct_name)));
                        }

                        let source_construct = match self.constructs.get(&transfer_cargo.source_construct_name) {
                            None => return ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::CouldNotFindConstruct(transfer_cargo.source_construct_name.clone())),
                            Some(source_construct) => source_construct
                        };

                        let target_construct = match self.constructs.get(&transfer_cargo.target_construct_name) {
                            None => return ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::CouldNotFindConstruct(transfer_cargo.target_construct_name.clone())),
                            Some(target_construct) => target_construct
                        };

                        match source_construct.position.position() {
                            ConstructPositionStatus::Sector(source_position) => {
                                match target_construct.position.position() {
                                    ConstructPositionStatus::Sector(target_position) => {
                                        return ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::Denied(format!("Cannot transfer, because {} is docked at {:?} and {} is docked at {:?}; One need to be docked at the other.", transfer_cargo.source_construct_name, source_position, transfer_cargo.target_construct_name, target_position)));
                                    }
                                    ConstructPositionStatus::Docked(target_docked_at_name) => {
                                        if target_docked_at_name.ne(&transfer_cargo.source_construct_name) {
                                            return ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::Denied(format!("Cannot transfer, because {} is docked at {:?} and {} is docked at {:?}; One need to be docked at the other.", transfer_cargo.source_construct_name, source_position, transfer_cargo.target_construct_name, target_docked_at_name)));
                                        } else {}
                                    }
                                }
                            }
                            ConstructPositionStatus::Docked(source_docked_at_name) => {
                                if source_docked_at_name.ne(&transfer_cargo.target_construct_name) {
                                    return ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::Denied(format!("Cannot transfer, because {} is docked at {:?} and that is not {}.", transfer_cargo.source_construct_name, source_docked_at_name, transfer_cargo.target_construct_name)));
                                } else {}
                            }
                        }

                        let amount = match self.constructs.get_mut(&transfer_cargo.target_construct_name).unwrap().push_event(&ConstructEventType::Internal(InternalConstructEventType::RequestUnload(transfer_cargo.amount.clone()))) {
                            ConstructEvenReturnType::RequestUnloadProcessed(amount) => amount.clone(),
                            return_value => return ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::Denied(format!("Could not unload {:?} from {}, got this message {:?}", transfer_cargo.amount, transfer_cargo.target_construct_name, return_value)))
                        };

                        let amount = match self.constructs.get_mut(&transfer_cargo.source_construct_name).unwrap().push_event(&ConstructEventType::Internal(InternalConstructEventType::RequestLoad(Amount::new(transfer_cargo.amount.product().clone(), amount)))) {
                            ConstructEvenReturnType::RequestLoadProcessed(amount) => amount,
                            return_value => return ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::Denied(format!("Could not load {:?} from {}, got this message {:?}", transfer_cargo.amount, transfer_cargo.source_construct_name, return_value)))
                        };

                        ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::CargoTransfered(amount))
                    }
                }
            }
        }
    }

    fn get_sector_position(&self, construct_name: String) -> &ConstructPositionSector {
        match self.constructs.get(construct_name.as_str()).expect("Looked up a construct_name that does not exist anymore").position.position() {
            ConstructPositionStatus::Docked(docker_construct_name) => {
                self.get_sector_position(docker_construct_name.clone())
            }
            ConstructPositionStatus::Sector(sector_position) => sector_position
        }
    }

    fn move_to_sector(&mut self, of_move_to_sector: OfMove) -> MyLittleUniverseReturnValues {
        if !self.sectors.contains_key(&of_move_to_sector.sector_position) {
            return MyLittleUniverseReturnValues::CouldNotMoveToSector(format!("Target sector does not exist {:?}", of_move_to_sector.sector_position));
        }

        //Handling source target first.
        match self.constructs.get(&of_move_to_sector.construct_name) {
            Some(construct) => {
                let position = construct.position().position();
                match position {
                    ConstructPositionStatus::Docked(construct_name) => {
                        return MyLittleUniverseReturnValues::CouldNotMoveToSector(format!("Is docked at {}", construct_name));
                    }
                    ConstructPositionStatus::Sector(source_sector_position) => {
                        if source_sector_position.sector_position().eq(&of_move_to_sector.sector_position)
                            && of_move_to_sector.group_address.is_some()
                            && source_sector_position.group_address().eq(&of_move_to_sector.group_address.unwrap()) {
                            return MyLittleUniverseReturnValues::CouldNotMoveToSector(format!("Construct {:?} is already in target position {:?}", of_move_to_sector.construct_name, of_move_to_sector));
                        }

                        match self.sectors.get_mut(source_sector_position.sector_position()) {
                            Some(source_sector) => {
                                match source_sector.push_event(&SectorEventType::Internal(InternalSectorEventType::Leave(of_move_to_sector.construct_name.clone()))) {
                                    SectorEvenReturnType::Approved => {}
                                    Denied(message) => {
                                        return MyLittleUniverseReturnValues::CouldNotMoveToSector(format!("Could not leave sector {:?}, because {}", of_move_to_sector.sector_position, message));
                                    }
                                    _ => {
                                        panic!("Source sector did not return expected event. Event: {:?}, Source Sector: {:?}", of_move_to_sector, of_move_to_sector.sector_position);
                                    }
                                }
                            }
                            None => {
                                return MyLittleUniverseReturnValues::CouldNotMoveToSector(format!("Source sector does not exist {:?}", of_move_to_sector.sector_position));
                            }
                        }
                    }
                }
            }
            None => {
                return MyLittleUniverseReturnValues::CouldNotMoveToSector(format!("Construct does not exist {}", of_move_to_sector.construct_name));
            }
        }

        //Then handle target sector
        let group_id = match self.sectors.get_mut(&of_move_to_sector.sector_position) {
            Some(target_sector) => {
                if let Entered(group_id) = target_sector.push_event(&SectorEventType::Internal(InternalSectorEventType::Enter(of_move_to_sector.construct_name.clone(), of_move_to_sector.group_address))) {
                    group_id
                } else {
                    panic!("Constructs are in bad state; It is removed from one sector but not added to the new one. Construct_name: {}; Reason: Target did not accept construct entering.", of_move_to_sector.construct_name);
                }
            }
            None => {
                panic!("Constructs are in bad state; It is removed from one sector but not added to the new one. Construct_name: {}; Reason: Target were gone when needed.", of_move_to_sector.construct_name);
            }
        };

        //Handling construct
        self.constructs.get_mut(&of_move_to_sector.construct_name).unwrap()
            .push_event(&ConstructEventType::External(ExternalConstructEventType::ConstructPosition(ExternalConstructPositionEventType::EnterSector(
                ConstructPositionSector::new(of_move_to_sector.sector_position.clone(), group_id)
            ))));


        MyLittleUniverseReturnValues::MovedToSector(group_id)
    }

    pub fn request_execute_turn(&mut self) {
        if self.time.request_execute_turn() {
            for construct in self.constructs.values_mut() {
                construct.push_event(&ConstructEventType::Internal(InternalConstructEventType::ExecuteTurn(self.time.turn())));
            }
            self.time.push_event(&TimeEventType::Internal(InternalTimeEventType::ReadyForNextTurn));
        }
    }
}


#[cfg(test)]
mod tests_int {
    use std::collections::HashMap;

    use crate::{ExternalCommandReturnValues, ExternalCommands};
    use crate::construct::amount::Amount;
    use crate::construct::construct::{Construct, ConstructEvenReturnType, ConstructEventType, ExternalConstructEventType, InternalConstructEventType};
    use crate::construct::construct_position::{ConstructPositionEventReturnType, ConstructPositionSector, ConstructPositionStatus, ExternalConstructPositionEventType};
    use crate::construct::construct_position::ConstructPositionStatus::{Docked, Sector};
    use crate::construct::production_module::ProductionModule;
    use crate::construct_module::ConstructModuleType::Production;
    use crate::my_little_universe::{ExternalUniverseEventType, MyLittleUniverse, MyLittleUniverseReturnValues, OfMove, OfTransferCargo};
    use crate::products::Product;
    use crate::sector::{ExternalSectorEventType, SectorEvenReturnType, SectorPosition};
    use crate::time::{ExternalTimeEventType, TimeEventReturnType, TimeStackState};
    use crate::universe_generator::generate_simple_universe;

    #[test]
    fn it_works() {
        //Setup universe
        let sector_position = ConstructPositionSector::new(SectorPosition::new(1, 1, 1), 0);
        let mut construct = Construct::new("The base".to_string(), 500, sector_position);
        let ore_production = ProductionModule::new(
            "PowerToOre".to_string(),
            vec![Amount::new(Product::PowerCells, 1)],
            vec![Amount::new(Product::Ores, 2)],
            1,
            0,
        );
        assert_eq!(Ok(()), construct.install(Production(ore_production.clone())));

        let mut constructs: HashMap<String, Construct> = HashMap::new();
        let construct_name = construct.name().to_string();
        constructs.insert(construct_name.clone(), construct);

        let mut universe = MyLittleUniverse::new("universe_name".to_string(), TimeStackState::new(), constructs, HashMap::new());

        //testing
        assert_eq!(
            ConstructEvenReturnType::RequestLoadProcessed(200),
            universe.constructs.get_mut(construct_name.as_str().clone()).unwrap().push_event(&ConstructEventType::Internal(InternalConstructEventType::RequestLoad(Amount::new(Product::PowerCells, 200))))
        );
        assert_eq!(
            ConstructEvenReturnType::RequestUnloadProcessed(0),
            universe.constructs.get_mut(construct_name.as_str().clone()).unwrap().push_event(&ConstructEventType::Internal(InternalConstructEventType::RequestUnload(Amount::new(Product::Ores, 2))))
        );

        assert_eq!(
            ExternalCommandReturnValues::Time(TimeEventReturnType::Received),
            universe.handle_event(ExternalCommands::Time(ExternalTimeEventType::StartUntilTurn(100)))
        );
        universe.request_execute_turn();
        universe.request_execute_turn();

        assert!(
            matches!(
                universe.handle_event(ExternalCommands::Construct("The base".to_string(), ExternalConstructEventType::GetConstructState{include_stack: false})),
                ExternalCommandReturnValues::Construct(ConstructEvenReturnType::ConstructState{..})
            )
        );

        assert_eq!(
            ConstructEvenReturnType::RequestUnloadProcessed(2),
            universe.constructs.get_mut(construct_name.as_str().clone()).unwrap().push_event(&ConstructEventType::Internal(InternalConstructEventType::RequestUnload(Amount::new(Product::Ores, 2))))
        );

        assert_eq!(
            ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::CouldNotFindConstruct("!The base".to_string())),
            universe.handle_event(ExternalCommands::Construct("!The base".to_string(), ExternalConstructEventType::GetConstructState { include_stack: false })),
        );
    }

    #[test]
    fn move_sectors() {
        let mut universe = generate_simple_universe("the_universe".to_string());

        if let ExternalCommandReturnValues::Construct(ConstructEvenReturnType::ConstructState(construct)) = universe.handle_event(ExternalCommands::Construct("transport".to_string(), ExternalConstructEventType::GetConstructState { include_stack: false })) {
            assert_eq!(&Sector(ConstructPositionSector::new(SectorPosition::new(1, 1, 1), 0)), construct.position.position());
        } else {
            assert!(false);
        }

        assert_eq!(
            ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::CouldNotMoveToSector("Construct \"transport\" is already in target position OfMove { construct_name: \"transport\", sector_position: SectorPosition { x: 1, y: 1, z: 1 }, group_address: Some(0) }".to_string())),
            universe.handle_event(ExternalCommands::Universe(ExternalUniverseEventType::Move(OfMove::new("transport".to_string(), SectorPosition::new(1, 1, 1), Some(0))))),
        );

        assert_eq!(
            ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::CouldNotMoveToSector("Target sector does not exist SectorPosition { x: 3, y: 3, z: 3 }".to_string())),
            universe.handle_event(ExternalCommands::Universe(ExternalUniverseEventType::Move(OfMove::new("transport".to_string(), SectorPosition::new(3, 3, 3), None)))),
        );

        assert_eq!(
            ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::MovedToSector(1)),
            universe.handle_event(ExternalCommands::Universe(ExternalUniverseEventType::Move(OfMove::new("transport".to_string(), SectorPosition::new(2, 2, 2), None)))),
        );

        assert_eq!(
            ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::MovedToSector(1)),
            universe.handle_event(ExternalCommands::Universe(ExternalUniverseEventType::Move(OfMove::new("transport".to_string(), SectorPosition::new(1, 1, 1), None)))),
        );

        assert_eq!(
            ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::CouldNotMoveToSector("Construct \"transport\" is already in target position OfMove { construct_name: \"transport\", sector_position: SectorPosition { x: 1, y: 1, z: 1 }, group_address: Some(1) }".to_string())),
            universe.handle_event(ExternalCommands::Universe(ExternalUniverseEventType::Move(OfMove::new("transport".to_string(), SectorPosition::new(1, 1, 1), Some(1))))),
        );

        assert_eq!(
            ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::CouldNotMoveToSector("Target sector does not exist SectorPosition { x: 3, y: 3, z: 3 }".to_string())),
            universe.handle_event(ExternalCommands::Universe(ExternalUniverseEventType::Move(OfMove::new("transport".to_string(), SectorPosition::new(3, 3, 3), None)))),
        );
    }

    #[test]
    fn docking() {
        let mut universe = generate_simple_universe("the_universe".to_string());

        verify_all_constructs_position(&mut universe,
                                       Sector(ConstructPositionSector::new(SectorPosition::new(1, 1, 1), 0)),
                                       Sector(ConstructPositionSector::new(SectorPosition::new(1, 1, 1), 0)),
                                       Sector(ConstructPositionSector::new(SectorPosition::new(2, 2, 2), 0)),
        );

        assert_eq!(
            ExternalCommandReturnValues::Construct(ConstructEvenReturnType::ConstructPosition(ConstructPositionEventReturnType::Denied("Target has no free docking slots transport".to_string()))),
            universe.handle_event(ExternalCommands::Construct("The_base_1".to_string(), ExternalConstructEventType::ConstructPosition(ExternalConstructPositionEventType::Dock("transport".to_string())))),
        );

        verify_all_constructs_position(&mut universe,
                                       Sector(ConstructPositionSector::new(SectorPosition::new(1, 1, 1), 0)),
                                       Sector(ConstructPositionSector::new(SectorPosition::new(1, 1, 1), 0)),
                                       Sector(ConstructPositionSector::new(SectorPosition::new(2, 2, 2), 0)),
        );

        assert_eq!(
            ExternalCommandReturnValues::Construct(ConstructEvenReturnType::ConstructPosition(ConstructPositionEventReturnType::Denied("Construct \"transport\" is at position Sector(ConstructPositionSector { sector_position: SectorPosition { x: 1, y: 1, z: 1 }, group_address: 0 }) and \"The_base_2\" is at position Sector(ConstructPositionSector { sector_position: SectorPosition { x: 2, y: 2, z: 2 }, group_address: 0 }), but they need to be at the same position to dock.".to_string()))),
            universe.handle_event(ExternalCommands::Construct("transport".to_string(), ExternalConstructEventType::ConstructPosition(ExternalConstructPositionEventType::Dock("The_base_2".to_string())))),
        );

        verify_all_constructs_position(&mut universe,
                                       Sector(ConstructPositionSector::new(SectorPosition::new(1, 1, 1), 0)),
                                       Sector(ConstructPositionSector::new(SectorPosition::new(1, 1, 1), 0)),
                                       Sector(ConstructPositionSector::new(SectorPosition::new(2, 2, 2), 0)),
        );

        assert_eq!(
            ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::MovedToSector(1)),
            universe.handle_event(ExternalCommands::Universe(ExternalUniverseEventType::Move(OfMove::new("transport".to_string(), SectorPosition::new(2, 2, 2), None)))),
        );

        verify_all_constructs_position(&mut universe,
                                       Sector(ConstructPositionSector::new(SectorPosition::new(2, 2, 2), 1)),
                                       Sector(ConstructPositionSector::new(SectorPosition::new(1, 1, 1), 0)),
                                       Sector(ConstructPositionSector::new(SectorPosition::new(2, 2, 2), 0)),
        );

        assert_eq!(
            ExternalCommandReturnValues::Construct(ConstructEvenReturnType::ConstructPosition(ConstructPositionEventReturnType::Denied("Construct \"transport\" is at position Sector(ConstructPositionSector { sector_position: SectorPosition { x: 2, y: 2, z: 2 }, group_address: 1 }) and \"The_base_2\" is at position Sector(ConstructPositionSector { sector_position: SectorPosition { x: 2, y: 2, z: 2 }, group_address: 0 }), but they need to be at the same position to dock.".to_string()))),
            universe.handle_event(ExternalCommands::Construct("transport".to_string(), ExternalConstructEventType::ConstructPosition(ExternalConstructPositionEventType::Dock("The_base_2".to_string())))),
        );

        verify_all_constructs_position(&mut universe,
                                       Sector(ConstructPositionSector::new(SectorPosition::new(2, 2, 2), 1)),
                                       Sector(ConstructPositionSector::new(SectorPosition::new(1, 1, 1), 0)),
                                       Sector(ConstructPositionSector::new(SectorPosition::new(2, 2, 2), 0)),
        );

        assert_eq!(
            ExternalCommandReturnValues::Sector(SectorEvenReturnType::Entered(0)),
            universe.handle_event(ExternalCommands::Sector(SectorPosition::new(2, 2, 2), ExternalSectorEventType::MoveToGroup("transport".to_string(), Some(0)))),
        );

        verify_all_constructs_position(&mut universe,
                                       Sector(ConstructPositionSector::new(SectorPosition::new(2, 2, 2), 0)),
                                       Sector(ConstructPositionSector::new(SectorPosition::new(1, 1, 1), 0)),
                                       Sector(ConstructPositionSector::new(SectorPosition::new(2, 2, 2), 0)),
        );

        assert_eq!(
            ExternalCommandReturnValues::Construct(ConstructEvenReturnType::ConstructPosition(ConstructPositionEventReturnType::RequestProcessed)),
            universe.handle_event(ExternalCommands::Construct("transport".to_string(), ExternalConstructEventType::ConstructPosition(ExternalConstructPositionEventType::Dock("The_base_2".to_string())))),
        );


        verify_all_constructs_position(&mut universe,
                                       Docked("The_base_2".to_string()),
                                       Sector(ConstructPositionSector::new(SectorPosition::new(1, 1, 1), 0)),
                                       Sector(ConstructPositionSector::new(SectorPosition::new(2, 2, 2), 0)),
        );

        assert_eq!(
            ExternalCommandReturnValues::Construct(ConstructEvenReturnType::ConstructPosition(ConstructPositionEventReturnType::Denied("Construct transport is already docked at The_base_2 so cannot dock again. Use Undock first.".to_string()))),
            universe.handle_event(ExternalCommands::Construct("transport".to_string(), ExternalConstructEventType::ConstructPosition(ExternalConstructPositionEventType::Dock("The_base_2".to_string())))),
        );

        verify_all_constructs_position(&mut universe,
                                       Docked("The_base_2".to_string()),
                                       Sector(ConstructPositionSector::new(SectorPosition::new(1, 1, 1), 0)),
                                       Sector(ConstructPositionSector::new(SectorPosition::new(2, 2, 2), 0)),
        );

        assert_eq!(
            ExternalCommandReturnValues::Construct(ConstructEvenReturnType::ConstructPosition(ConstructPositionEventReturnType::RequestProcessed)),
            universe.handle_event(ExternalCommands::Construct("transport".to_string(), ExternalConstructEventType::ConstructPosition(ExternalConstructPositionEventType::Undock))),
        );

        verify_all_constructs_position(&mut universe,
                                       Sector(ConstructPositionSector::new(SectorPosition::new(2, 2, 2), 0)),
                                       Sector(ConstructPositionSector::new(SectorPosition::new(1, 1, 1), 0)),
                                       Sector(ConstructPositionSector::new(SectorPosition::new(2, 2, 2), 0)),
        );

        assert_eq!(
            ExternalCommandReturnValues::Construct(ConstructEvenReturnType::ConstructPosition(ConstructPositionEventReturnType::Denied("Cannot undock because is not docked. Is in sector ConstructPositionSector { sector_position: SectorPosition { x: 2, y: 2, z: 2 }, group_address: 0 }".to_string()))),
            universe.handle_event(ExternalCommands::Construct("transport".to_string(), ExternalConstructEventType::ConstructPosition(ExternalConstructPositionEventType::Undock))),
        );

        verify_all_constructs_position(&mut universe,
                                       Sector(ConstructPositionSector::new(SectorPosition::new(2, 2, 2), 0)),
                                       Sector(ConstructPositionSector::new(SectorPosition::new(1, 1, 1), 0)),
                                       Sector(ConstructPositionSector::new(SectorPosition::new(2, 2, 2), 0)),
        );
        assert_eq!(
            ExternalCommandReturnValues::Construct(ConstructEvenReturnType::ConstructPosition(ConstructPositionEventReturnType::RequestProcessed)),
            universe.handle_event(ExternalCommands::Construct("transport".to_string(), ExternalConstructEventType::ConstructPosition(ExternalConstructPositionEventType::Dock("The_base_2".to_string())))),
        );

        verify_all_constructs_position(&mut universe,
                                       Docked("The_base_2".to_string()),
                                       Sector(ConstructPositionSector::new(SectorPosition::new(1, 1, 1), 0)),
                                       Sector(ConstructPositionSector::new(SectorPosition::new(2, 2, 2), 0)),
        );

        assert_eq!(
            ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::MovedToSector(0)),
            universe.handle_event(ExternalCommands::Universe(ExternalUniverseEventType::Move(OfMove::new("The_base_2".to_string(), SectorPosition::new(1, 1, 1), Some(0))))),
        );

        verify_all_constructs_position(&mut universe,
                                       Docked("The_base_2".to_string()),
                                       Sector(ConstructPositionSector::new(SectorPosition::new(1, 1, 1), 0)),
                                       Sector(ConstructPositionSector::new(SectorPosition::new(1, 1, 1), 0)),
        );

        assert_eq!(
            ExternalCommandReturnValues::Construct(ConstructEvenReturnType::ConstructPosition(ConstructPositionEventReturnType::RequestProcessed)),
            universe.handle_event(ExternalCommands::Construct("The_base_2".to_string(), ExternalConstructEventType::ConstructPosition(ExternalConstructPositionEventType::Dock("The_base_1".to_string())))),
        );

        verify_all_constructs_position(&mut universe,
                                       Docked("The_base_2".to_string()),
                                       Sector(ConstructPositionSector::new(SectorPosition::new(1, 1, 1), 0)),
                                       Docked("The_base_1".to_string()),
        );

        assert_eq!(
            ExternalCommandReturnValues::Construct(ConstructEvenReturnType::ConstructPosition(ConstructPositionEventReturnType::Denied("Construct The_base_1 is already docked at The_base_2 or one of its docker parents.".to_string()))),
            universe.handle_event(ExternalCommands::Construct("The_base_1".to_string(), ExternalConstructEventType::ConstructPosition(ExternalConstructPositionEventType::Dock("The_base_2".to_string())))),
        );

        verify_all_constructs_position(&mut universe,
                                       Docked("The_base_2".to_string()),
                                       Sector(ConstructPositionSector::new(SectorPosition::new(1, 1, 1), 0)),
                                       Docked("The_base_1".to_string()),
        );
    }

    #[test]
    fn transfering() {
        let mut universe = generate_simple_universe("the_universe".to_string());

        assert_eq!(
            ConstructEvenReturnType::RequestLoadProcessed(200),
            universe.constructs.get_mut("The_base_1").unwrap().push_event(&ConstructEventType::Internal(InternalConstructEventType::RequestLoad(Amount::new(Product::PowerCells, 200))))
        );

        assert_eq!(
            ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::Denied("Cannot transfer because source transport and target transport construct is the same".to_string())),
            universe.handle_event(ExternalCommands::Universe(ExternalUniverseEventType::TransferCargo(OfTransferCargo::new(
                "transport".to_string(),
                "transport".to_string(),
                Amount::new(Product::Metals, 100),
            )))),
        );

        assert_eq!(
            ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::Denied("Cannot transfer, because transport is docked at ConstructPositionSector { sector_position: SectorPosition { x: 1, y: 1, z: 1 }, group_address: 0 } and The_base_1 is docked at ConstructPositionSector { sector_position: SectorPosition { x: 1, y: 1, z: 1 }, group_address: 0 }; One need to be docked at the other.".to_string())),
            universe.handle_event(ExternalCommands::Universe(ExternalUniverseEventType::TransferCargo(OfTransferCargo::new(
                "transport".to_string(),
                "The_base_1".to_string(),
                Amount::new(Product::Metals, 100),
            )))),
        );

        assert_eq!(
            ExternalCommandReturnValues::Construct(ConstructEvenReturnType::ConstructPosition(ConstructPositionEventReturnType::RequestProcessed)),
            universe.handle_event(ExternalCommands::Construct("transport".to_string(), ExternalConstructEventType::ConstructPosition(ExternalConstructPositionEventType::Dock("The_base_1".to_string())))),
        );

        assert_eq!(
            ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::CargoTransfered(0)),
            universe.handle_event(ExternalCommands::Universe(ExternalUniverseEventType::TransferCargo(OfTransferCargo::new(
                "transport".to_string(),
                "The_base_1".to_string(),
                Amount::new(Product::Metals, 100),
            )))),
        );

        assert_eq!(
            ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::CargoTransfered(50)),
            universe.handle_event(ExternalCommands::Universe(ExternalUniverseEventType::TransferCargo(OfTransferCargo::new(
                "transport".to_string(),
                "The_base_1".to_string(),
                Amount::new(Product::PowerCells, 50),
            )))),
        );

        assert_eq!(
            ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::CargoTransfered(150)),
            universe.handle_event(ExternalCommands::Universe(ExternalUniverseEventType::TransferCargo(OfTransferCargo::new(
                "transport".to_string(),
                "The_base_1".to_string(),
                Amount::new(Product::PowerCells, 200),
            )))),
        );

        assert_eq!(
            ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::CargoTransfered(0)),
            universe.handle_event(ExternalCommands::Universe(ExternalUniverseEventType::TransferCargo(OfTransferCargo::new(
                "transport".to_string(),
                "The_base_1".to_string(),
                Amount::new(Product::PowerCells, 200),
            )))),
        );

        assert_eq!(
            ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::CargoTransfered(200)),
            universe.handle_event(ExternalCommands::Universe(ExternalUniverseEventType::TransferCargo(OfTransferCargo::new(
                "The_base_1".to_string(),
                "transport".to_string(),
                Amount::new(Product::PowerCells, 300),
            )))),
        );

        assert_eq!(
            ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::CargoTransfered(0)),
            universe.handle_event(ExternalCommands::Universe(ExternalUniverseEventType::TransferCargo(OfTransferCargo::new(
                "The_base_1".to_string(),
                "transport".to_string(),
                Amount::new(Product::PowerCells, 300),
            )))),
        );
    }

    fn verify_all_constructs_position(universe: &mut MyLittleUniverse, transport_position: ConstructPositionStatus, base_1_position: ConstructPositionStatus, base_2_position: ConstructPositionStatus) {
        if let ExternalCommandReturnValues::Construct(ConstructEvenReturnType::ConstructState(construct)) = universe.handle_event(ExternalCommands::Construct("transport".to_string(), ExternalConstructEventType::GetConstructState { include_stack: false })) {
            assert_eq!(&transport_position, construct.position.position());
        } else {
            assert!(false);
        }

        if let ExternalCommandReturnValues::Construct(ConstructEvenReturnType::ConstructState(construct)) = universe.handle_event(ExternalCommands::Construct("The_base_1".to_string(), ExternalConstructEventType::GetConstructState { include_stack: false })) {
            assert_eq!(&base_1_position, construct.position.position());
        } else {
            assert!(false);
        }

        if let ExternalCommandReturnValues::Construct(ConstructEvenReturnType::ConstructState(construct)) = universe.handle_event(ExternalCommands::Construct("The_base_2".to_string(), ExternalConstructEventType::GetConstructState { include_stack: false })) {
            assert_eq!(&base_2_position, construct.position.position());
        } else {
            assert!(false);
        }
    }
}