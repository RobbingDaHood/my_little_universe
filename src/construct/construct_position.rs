use serde::{Deserialize, Serialize};

use crate::construct::construct::Construct;
use crate::construct::construct_position::ConstructPositionEventReturnType::{Denied, RequestProcessed};
use crate::construct::construct_position::ConstructPositionStatus::{IsDocked, InSector};
use crate::my_little_universe::MyLittleUniverse;
use crate::sector::{InternalSectorEventType, SectorEvenReturnType, SectorEventType, SectorPosition};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ConstructPositionStatus {
    IsDocked(String),
    InSector(ConstructPositionSector),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ConstructPositionSector {
    sector_position: SectorPosition,
    group_address: usize,
}

impl ConstructPositionSector {
    pub fn new(sector_position: SectorPosition, group_address: usize) -> Self {
        ConstructPositionSector { sector_position, group_address }
    }

    pub fn sector_position(&self) -> &SectorPosition {
        &self.sector_position
    }
    pub fn group_address(&self) -> usize {
        self.group_address
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ConstructPositionEventType {
    Internal(InternalConstructPositionEventType),
    External(ExternalConstructPositionEventType),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ExternalConstructPositionEventType {
    Dock(String),
    Undock,
    EnterSector(ConstructPositionSector),
    //TODO there should just be one move external method and the rest should be internal
    EnterGroup(usize),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum InternalConstructPositionEventType {
    Undock(ConstructPositionSector),
    Undocked(String),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ConstructPositionEventReturnType {
    RequestProcessed,
    Denied(String),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ConstructPositionState {
    position: ConstructPositionStatus,
    source_construct_name: String,
    docker_modules: Vec<DockerModule>,
}

impl ConstructPositionState {
    pub fn new(source_construct_name: String, sector_position: ConstructPositionSector) -> Self {
        ConstructPositionState { position: InSector(sector_position), source_construct_name, docker_modules: Vec::new() }
    }
    pub fn position(&self) -> &ConstructPositionStatus {
        &self.position
    }
    pub fn source_construct_name(&self) -> &str {
        &self.source_construct_name
    }
    pub fn docker_modules(&self) -> &Vec<DockerModule> {
        &self.docker_modules
    }

    pub fn handle_event(&mut self, event: &ConstructPositionEventType) -> ConstructPositionEventReturnType {
        match event {
            ConstructPositionEventType::External(ExternalConstructPositionEventType::Dock(construct_name)) => {
                if self.source_construct_name.eq(construct_name) {
                    return Denied("Construct cannot dock with itself.".to_string());
                }
                self.position = IsDocked(construct_name.clone());
                RequestProcessed
            }
            ConstructPositionEventType::External(ExternalConstructPositionEventType::Undock) => {
                Denied("External Undock should never hit construct, use internal dock instead that contains all relevant information".to_string())
            }
            ConstructPositionEventType::External(ExternalConstructPositionEventType::EnterSector(sector_position)) => {
                self.position = InSector(sector_position.clone());
                RequestProcessed
            }
            ConstructPositionEventType::External(ExternalConstructPositionEventType::EnterGroup(group_address)) => {
                match self.position() {
                    ConstructPositionStatus::InSector(current_position) => {
                        self.position = InSector(ConstructPositionSector::new(current_position.sector_position.clone(), group_address.clone()));
                        RequestProcessed
                    }
                    ConstructPositionStatus::IsDocked(docked_at) => Denied(format!("Currently docket at {} so cannot update the group for {}.", docked_at, self.source_construct_name))
                }
            }
            ConstructPositionEventType::Internal(InternalConstructPositionEventType::Undock(sector_position)) => {
                self.position = InSector(sector_position.clone());
                RequestProcessed
            }
            ConstructPositionEventType::Internal(InternalConstructPositionEventType::Undocked(construct_name)) => {
                self.docker_modules.iter_mut()
                    .find(|m| {
                        match m.docked_construct() {
                            Some(docked_construct_name) => docked_construct_name.eq(construct_name),
                            None => false
                        }
                    })
                    .expect("Trying to undock construct that is not docked")
                    .undock();
                RequestProcessed
            }
        }
    }

    pub fn install(&mut self) {
        self.docker_modules.push(DockerModule::new());
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DockerModule {
    docked_construct: Option<String>,
}

impl DockerModule {
    pub fn new() -> Self {
        DockerModule { docked_construct: None }
    }

    pub fn docked_construct(&self) -> &Option<String> {
        &self.docked_construct
    }

    pub fn undock(&mut self) {
        self.docked_construct = None;
    }
}

impl Construct {
    pub fn handle_docking_request(&mut self, source_construct_name: String) -> ConstructPositionEventReturnType {
        let free_docking_slot = self.position.docker_modules.iter_mut().find(|dm| dm.docked_construct.is_none());
        match free_docking_slot {
            None => return ConstructPositionEventReturnType::Denied(format!("Target has no free docking slots {}", self.name())),
            Some(slot) => slot.docked_construct = Some(source_construct_name)
        }

        ConstructPositionEventReturnType::RequestProcessed
    }

    pub fn handle_docked(&mut self, target_construct_name: String) -> ConstructPositionEventReturnType {
        self.position.position = ConstructPositionStatus::IsDocked(target_construct_name);
        ConstructPositionEventReturnType::RequestProcessed
    }
}

impl MyLittleUniverse {
    pub fn handle_docking_request(&mut self, source_construct_name: String, target_construct_name: String) -> ConstructPositionEventReturnType {
        //Validation
        if source_construct_name.eq(&target_construct_name) {
            return Denied("Construct cannot dock with itself.".to_string());
        }

        let (source_construct, position) = match self.constructs().get(source_construct_name.as_str()) {
            None => return ConstructPositionEventReturnType::Denied(format!("No construct with the name {}", source_construct_name)),
            Some(source_construct) => {
                match &source_construct.position.position {
                    IsDocked(docked_at_name) => return ConstructPositionEventReturnType::Denied(format!("Construct {} is already docked at {} so cannot dock again. Use Undock first.", source_construct_name, docked_at_name)),
                    InSector(position) => (source_construct.clone(), position.clone())
                }
            }
        };

        let target_construct = match self.constructs().get(target_construct_name.as_str()) {
            None => return ConstructPositionEventReturnType::Denied(format!("No construct with the name {}", target_construct_name)),
            Some(construct) => {
                if self.construct_is_part_of_docker_parents(target_construct_name.clone(), &source_construct_name) {
                    return Denied(format!("Construct {} is already docked at {} or one of its docker parents.", source_construct_name, target_construct_name));
                }
                construct.clone()
            }
        };

        if target_construct.position.position.ne(source_construct.position().position()) {
            return Denied(format!("Construct {:?} is at position {:?} and {:?} is at position {:?}, but they need to be at the same position to dock.",
                                  source_construct_name,
                                  source_construct.position.position,
                                  target_construct_name,
                                  target_construct.position().position()));
        }

        //Mutation
        match self.constructs.get_mut(target_construct_name.as_str()).unwrap().handle_docking_request(source_construct_name.clone()) {
            ConstructPositionEventReturnType::RequestProcessed => {
                self.constructs.get_mut(source_construct_name.as_str()).unwrap().handle_docked(target_construct_name);
            }
            ConstructPositionEventReturnType::Denied(error) => return ConstructPositionEventReturnType::Denied(error)
        };

        match self.sectors.get_mut(position.sector_position()) {
            None => return ConstructPositionEventReturnType::Denied(format!("No sector at position {:?}", position)),
            Some(sector) => {
                match sector.push_event(&SectorEventType::Internal(InternalSectorEventType::Leave(source_construct.name().to_string(), position.group_address))) {
                    SectorEvenReturnType::Approved => {}
                    other_responses => return ConstructPositionEventReturnType::Denied(format!("Construct {} could not leave {:?}, got this response {:?}", source_construct.name(), position, other_responses))
                }
            }
        }

        RequestProcessed
    }

    fn construct_is_part_of_docker_parents(&self, first_docked_construct_name: String, query_construct_name: &String) -> bool {
        if first_docked_construct_name.eq(query_construct_name) {
            return true;
        } else {
            match self.constructs.get(first_docked_construct_name.as_str()).expect("Looked up a construct_name that does not exist anymore").position.position() {
                ConstructPositionStatus::IsDocked(docker_construct_name) => {
                    self.construct_is_part_of_docker_parents(docker_construct_name.clone(), query_construct_name)
                }
                ConstructPositionStatus::InSector(_) => return false
            }
        }
    }
}


#[cfg(test)]
mod tests_int {
    use std::collections::HashMap;

    use crate::construct::construct::Construct;
    use crate::construct::construct_position::{ConstructPositionEventReturnType, ConstructPositionEventType, ConstructPositionSector, ConstructPositionState, ConstructPositionStatus, ExternalConstructPositionEventType, InternalConstructPositionEventType};
    use crate::construct::construct_position::ConstructPositionEventReturnType::{Denied, RequestProcessed};
    use crate::construct::construct_position::ConstructPositionStatus::{IsDocked, InSector};
    use crate::my_little_universe::MyLittleUniverse;
    use crate::sector::{Sector, SectorPosition};
    use crate::time::TimeStackState;

    #[test]
    fn docking_module() {
        let sector_position = ConstructPositionSector::new(SectorPosition::new(1, 1, 1), 0);
        let mut position1 = ConstructPositionState::new("FirstLocation1".to_string(), sector_position.clone());
        let position2 = ConstructPositionState::new("FirstLocation2".to_string(), sector_position.clone());

        assert_eq!(InSector(sector_position.clone()), *position1.position());
        assert_eq!(InSector(sector_position.clone()), *position2.position());
        assert_eq!(
            ConstructPositionEventReturnType::Denied("Construct cannot dock with itself.".to_string()),
            position1.handle_event(&ConstructPositionEventType::External(ExternalConstructPositionEventType::Dock(position1.source_construct_name.to_string())))
        );
        assert_eq!(InSector(sector_position.clone()), *position1.position());
        assert_eq!(InSector(sector_position.clone()), *position2.position());

        assert_eq!(
            ConstructPositionEventReturnType::RequestProcessed,
            position1.handle_event(&ConstructPositionEventType::External(ExternalConstructPositionEventType::Dock(position2.source_construct_name.to_string())))
        );
        assert_eq!(IsDocked(position2.source_construct_name.clone()), *position1.position());
        assert_eq!(InSector(sector_position.clone()), *position2.position());

        assert_eq!(
            ConstructPositionEventReturnType::RequestProcessed,
            position1.handle_event(&ConstructPositionEventType::Internal(InternalConstructPositionEventType::Undock(sector_position.clone())))
        );
        assert_eq!(InSector(sector_position.clone()), *position1.position());
        assert_eq!(InSector(sector_position.clone()), *position2.position());

        assert_eq!(
            ConstructPositionEventReturnType::Denied("External Undock should never hit construct, use internal dock instead that contains all relevant information".to_string()),
            position1.handle_event(&ConstructPositionEventType::External(ExternalConstructPositionEventType::Undock))
        );
        assert_eq!(InSector(sector_position.clone()), *position1.position());
        assert_eq!(InSector(sector_position.clone()), *position2.position());
    }

    #[test]
    fn docking_universe() {
        let the_base1_name = "The base1";
        let the_base2_name = "The base2";
        let sector_position = ConstructPositionSector::new(SectorPosition::new(1, 1, 1), 0);
        let construct1 = Construct::new(the_base1_name.to_string(), 500, sector_position.clone());
        let mut construct2 = Construct::new(the_base2_name.to_string(), 500, sector_position.clone());
        construct2.position.install();
        let mut constructs: HashMap<String, Construct> = HashMap::new();
        constructs.insert(construct1.name().to_string(), construct1);
        constructs.insert(construct2.name().to_string(), construct2);

        let mut sectors = HashMap::new();
        sectors.insert(sector_position.sector_position.clone(), Sector::new(Vec::new(), sector_position.sector_position.clone()));

        let mut universe = MyLittleUniverse::new("universe_name".to_string(), TimeStackState::new(), constructs, sectors);

        assert_eq!(InSector(sector_position.clone()), *universe.constructs.get(the_base1_name).unwrap().position().position());
        assert_eq!(InSector(sector_position.clone()), *universe.constructs.get(the_base2_name).unwrap().position().position());
        assert_eq!(None, universe.constructs.get(the_base2_name).unwrap().position().docker_modules.get(0).unwrap().docked_construct);

        assert_eq!(
            ConstructPositionEventReturnType::Denied("Construct cannot dock with itself.".to_string()),
            universe.handle_docking_request(the_base1_name.to_string(), the_base1_name.to_string())
        );

        assert_eq!(InSector(sector_position.clone()), *universe.constructs.get(the_base1_name).unwrap().position().position());
        assert_eq!(InSector(sector_position.clone()), *universe.constructs.get(the_base2_name).unwrap().position().position());
        assert_eq!(None, universe.constructs.get(the_base2_name).unwrap().position().docker_modules.get(0).unwrap().docked_construct);

        assert_eq!(
            ConstructPositionEventReturnType::RequestProcessed,
            universe.handle_docking_request(the_base1_name.to_string(), the_base2_name.to_string())
        );

        assert_eq!(ConstructPositionStatus::IsDocked(the_base2_name.to_string()), *universe.constructs.get(the_base1_name).unwrap().position().position());
        assert_eq!(InSector(sector_position.clone()), *universe.constructs.get(the_base2_name).unwrap().position().position());
        assert_eq!(Some(the_base1_name.to_string()), universe.constructs.get(the_base2_name).unwrap().position().docker_modules.get(0).unwrap().docked_construct);

        assert_eq!(
            ConstructPositionEventReturnType::Denied("Construct The base2 is already docked at The base1 or one of its docker parents.".to_string()),
            universe.handle_docking_request(the_base2_name.to_string(), the_base1_name.to_string())
        );

        assert_eq!(ConstructPositionStatus::IsDocked(the_base2_name.to_string()), *universe.constructs.get(the_base1_name).unwrap().position().position());
        assert_eq!(InSector(sector_position.clone()), *universe.constructs.get(the_base2_name).unwrap().position().position());
        assert_eq!(Some(the_base1_name.to_string()), universe.constructs.get(the_base2_name).unwrap().position().docker_modules.get(0).unwrap().docked_construct);

        assert_eq!(
            Denied("External Undock should never hit construct, use internal dock instead that contains all relevant information".to_string()),
            universe.constructs.get_mut(the_base1_name).unwrap().position.handle_event(&ConstructPositionEventType::External(ExternalConstructPositionEventType::Undock))
        );

        assert_eq!(
            RequestProcessed,
            universe.constructs.get_mut(the_base1_name).unwrap().position.handle_event(&ConstructPositionEventType::Internal(InternalConstructPositionEventType::Undock(sector_position.clone())))
        );

        assert_eq!(InSector(sector_position.clone()), *universe.constructs.get(the_base1_name).unwrap().position().position());
        assert_eq!(InSector(sector_position.clone()), *universe.constructs.get(the_base2_name).unwrap().position().position());
        assert_eq!(Some(the_base1_name.to_string()), universe.constructs.get(the_base2_name).unwrap().position().docker_modules.get(0).unwrap().docked_construct);

        assert_eq!(
            RequestProcessed,
            universe.constructs.get_mut(the_base2_name).unwrap().position.handle_event(&ConstructPositionEventType::Internal(InternalConstructPositionEventType::Undocked(the_base1_name.to_string())))
        );

        assert_eq!(InSector(sector_position.clone()), *universe.constructs.get(the_base1_name).unwrap().position().position());
        assert_eq!(InSector(sector_position.clone()), *universe.constructs.get(the_base2_name).unwrap().position().position());
        assert_eq!(None, universe.constructs.get(the_base2_name).unwrap().position().docker_modules.get(0).unwrap().docked_construct);

        assert_eq!(
            ConstructPositionEventReturnType::Denied("Target has no free docking slots The base1".to_string()),
            universe.handle_docking_request(the_base2_name.to_string(), the_base1_name.to_string())
        );

        assert_eq!(InSector(sector_position.clone()), *universe.constructs.get(the_base1_name).unwrap().position().position());
        assert_eq!(InSector(sector_position.clone()), *universe.constructs.get(the_base2_name).unwrap().position().position());
        assert_eq!(None, universe.constructs.get(the_base2_name).unwrap().position().docker_modules.get(0).unwrap().docked_construct);
    }
}
