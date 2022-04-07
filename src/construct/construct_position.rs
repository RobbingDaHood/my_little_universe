use serde::{Deserialize, Serialize};

use crate::construct::construct::Construct;
use crate::construct::construct_position::ConstructPosition::{Docked, Sector};
use crate::construct::construct_position::ConstructPositionEventReturnType::{Denied, RequestProcessed};
use crate::my_little_universe::MyLittleUniverse;
use crate::sector::SectorPosition;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ConstructPosition {
    Docked(String),
    Sector(SectorPosition),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ExternalConstructPositionEventType {
    Dock(String),
    Undock(SectorPosition),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ConstructPositionEventReturnType {
    RequestProcessed,
    Denied(String),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ConstructPositionState {
    position: ConstructPosition,
    source_construct_name: String,
    docker_modules: Vec<DockerModule>,
}

impl ConstructPositionState {
    pub fn new(source_construct_name: String, sector_position: SectorPosition) -> Self {
        ConstructPositionState { position: Sector(sector_position), source_construct_name, docker_modules: Vec::new() }
    }
    pub fn position(&self) -> &ConstructPosition {
        &self.position
    }

    pub fn handle_event(&mut self, event: &ExternalConstructPositionEventType) -> ConstructPositionEventReturnType {
        match event {
            ExternalConstructPositionEventType::Dock(construct_name) => {
                if self.source_construct_name.eq(construct_name) {
                    return Denied("Construct cannot dock with itself.".to_string());
                }
                self.position = Docked(construct_name.clone());
                RequestProcessed
            }
            ExternalConstructPositionEventType::Undock(sector_position) => {
                self.position = Sector(sector_position.clone());
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
}

impl Construct {
    pub fn handle_docking_request(&mut self, source_construct_name: String) -> ConstructPositionEventReturnType {
        if let ConstructPosition::Docked(_) = &self.position.position {
            return ConstructPositionEventReturnType::Denied(format!("Cannot dock at target that itself is already docked {}", self.name()));
        }

        let free_docking_slot = self.position.docker_modules.iter_mut().find(|dm| dm.docked_construct.is_none());
        match free_docking_slot {
            None => return ConstructPositionEventReturnType::Denied(format!("Target has no free docking slots {}", self.name())),
            Some(slot) => slot.docked_construct = Some(source_construct_name)
        }

        ConstructPositionEventReturnType::RequestProcessed
    }

    pub fn handle_docked(&mut self, target_construct_name: String) -> ConstructPositionEventReturnType {
        self.position.position = ConstructPosition::Docked(target_construct_name);
        ConstructPositionEventReturnType::RequestProcessed
    }
}

impl MyLittleUniverse {
    pub fn handle_docking_request(&mut self, source_construct_name: String, target_construct_name: String) -> ConstructPositionEventReturnType {
        if source_construct_name.eq(&target_construct_name) {
            return Denied("Construct cannot dock with itself.".to_string());
        }

        if !self.constructs().contains_key(target_construct_name.as_str()) {
            return ConstructPositionEventReturnType::Denied(format!("No construct with the name {}", target_construct_name));
        };

        if !self.constructs().contains_key(source_construct_name.as_str()) {
            return ConstructPositionEventReturnType::Denied(format!("No construct with the name {}", target_construct_name));
        };

        return match self.constructs.get_mut(target_construct_name.as_str()).unwrap().handle_docking_request(source_construct_name.clone()) {
            ConstructPositionEventReturnType::RequestProcessed => self.constructs.get_mut(source_construct_name.as_str()).unwrap().handle_docked(target_construct_name),
            ConstructPositionEventReturnType::Denied(error) => ConstructPositionEventReturnType::Denied(error)
        };
    }
}


#[cfg(test)]
mod tests_int {
    use std::collections::HashMap;

    use crate::construct::construct::Construct;
    use crate::construct::construct_position::{ConstructPosition, ConstructPositionEventReturnType, ConstructPositionState, ExternalConstructPositionEventType};
    use crate::construct::construct_position::ConstructPosition::{Docked, Sector};
    use crate::construct::construct_position::ConstructPositionEventReturnType::RequestProcessed;
    use crate::my_little_universe::MyLittleUniverse;
    use crate::sector::SectorPosition;
    use crate::time::TimeStackState;

    #[test]
    fn docking_module() {
        let sector_position = SectorPosition::new(1, 1, 1);
        let mut position1 = ConstructPositionState::new("FirstLocation1".to_string(), sector_position.clone());
        let position2 = ConstructPositionState::new("FirstLocation2".to_string(), sector_position.clone());

        assert_eq!(Sector(sector_position.clone()), *position1.position());
        assert_eq!(Sector(sector_position.clone()), *position2.position());
        assert_eq!(
            ConstructPositionEventReturnType::Denied("Construct cannot dock with itself.".to_string()),
            position1.handle_event(&ExternalConstructPositionEventType::Dock(position1.source_construct_name.to_string()))
        );
        assert_eq!(Sector(sector_position.clone()), *position1.position());
        assert_eq!(Sector(sector_position.clone()), *position2.position());

        assert_eq!(
            ConstructPositionEventReturnType::RequestProcessed,
            position1.handle_event(&ExternalConstructPositionEventType::Dock(position2.source_construct_name.to_string()))
        );
        assert_eq!(Docked(position2.source_construct_name.clone()), *position1.position());
        assert_eq!(Sector(sector_position.clone()), *position2.position());

        assert_eq!(
            ConstructPositionEventReturnType::RequestProcessed,
            position1.handle_event(&ExternalConstructPositionEventType::Undock(sector_position.clone()))
        );
        assert_eq!(Sector(sector_position.clone()), *position1.position());
        assert_eq!(Sector(sector_position.clone()), *position2.position());
    }

    #[test]
    fn docking_universe() {
        let the_base1_name = "The base1";
        let the_base2_name = "The base2";
        let sector_position = SectorPosition::new(1, 1, 1);
        let construct1 = Construct::new(the_base1_name.to_string(), 500, sector_position.clone());
        let mut construct2 = Construct::new(the_base2_name.to_string(), 500, sector_position.clone());
        construct2.position.install();
        let mut constructs: HashMap<String, Construct> = HashMap::new();
        constructs.insert(construct1.name().to_string(), construct1);
        constructs.insert(construct2.name().to_string(), construct2);
        let mut universe = MyLittleUniverse::new("universe_name".to_string(), TimeStackState::new(), constructs, HashMap::new());

        assert_eq!(Sector(sector_position.clone()), *universe.constructs.get(the_base1_name).unwrap().position().position());
        assert_eq!(Sector(sector_position.clone()), *universe.constructs.get(the_base2_name).unwrap().position().position());

        assert_eq!(
            ConstructPositionEventReturnType::Denied("Construct cannot dock with itself.".to_string()),
            universe.handle_docking_request(the_base1_name.to_string(), the_base1_name.to_string())
        );

        assert_eq!(Sector(sector_position.clone()), *universe.constructs.get(the_base1_name).unwrap().position().position());
        assert_eq!(Sector(sector_position.clone()), *universe.constructs.get(the_base2_name).unwrap().position().position());

        assert_eq!(
            ConstructPositionEventReturnType::RequestProcessed,
            universe.handle_docking_request(the_base1_name.to_string(), the_base2_name.to_string())
        );

        assert_eq!(ConstructPosition::Docked(the_base2_name.to_string()), *universe.constructs.get(the_base1_name).unwrap().position().position());
        assert_eq!(Sector(sector_position.clone()), *universe.constructs.get(the_base2_name).unwrap().position().position());

        assert_eq!(
            ConstructPositionEventReturnType::Denied("Cannot dock at target that itself is already docked The base1".to_string()),
            universe.handle_docking_request(the_base2_name.to_string(), the_base1_name.to_string())
        );

        assert_eq!(ConstructPosition::Docked(the_base2_name.to_string()), *universe.constructs.get(the_base1_name).unwrap().position().position());
        assert_eq!(Sector(sector_position.clone()), *universe.constructs.get(the_base2_name).unwrap().position().position());

        assert_eq!(
            RequestProcessed,
            universe.constructs.get_mut(the_base1_name).unwrap().position.handle_event(&ExternalConstructPositionEventType::Undock(sector_position.clone()))
        );

        assert_eq!(Sector(sector_position.clone()), *universe.constructs.get(the_base1_name).unwrap().position().position());
        assert_eq!(Sector(sector_position.clone()), *universe.constructs.get(the_base2_name).unwrap().position().position());

        assert_eq!(
            ConstructPositionEventReturnType::Denied("Target has no free docking slots The base1".to_string()),
            universe.handle_docking_request(the_base2_name.to_string(), the_base1_name.to_string())
        );

        assert_eq!(Sector(sector_position.clone()), *universe.constructs.get(the_base1_name).unwrap().position().position());
        assert_eq!(Sector(sector_position.clone()), *universe.constructs.get(the_base2_name).unwrap().position().position());
    }
}
