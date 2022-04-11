use serde::{Deserialize, Serialize};

use crate::sector::SectorEvenReturnType::{Approved, Denied, Entered, SectorState};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum SectorEventType {
    Internal(InternalSectorEventType),
    External(ExternalSectorEventType),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum InternalSectorEventType {
    Leave(String),
    Enter(String, Option<usize>),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ExternalSectorEventType {
    GetSectorState,
    MoveToGroup(String, Option<usize>)
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum SectorEvenReturnType {
    Approved,
    Denied(String),
    SectorState(Sector),
    Entered(usize),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Sector {
    groups: Vec<Vec<String>>,
    position: SectorPosition,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Hash, std::cmp::Eq)]
pub struct SectorPosition {
    x: u8,
    y: u8,
    z: u8,
}

impl SectorPosition {
    pub fn new(x: u8, y: u8, z: u8) -> Self {
        SectorPosition { x, y, z }
    }
}

impl Sector {
    pub fn new(groups: Vec<Vec<String>>, position: SectorPosition) -> Self {
        Sector { groups, position }
    }
    pub fn groups(&self) -> &Vec<Vec<String>> {
        &self.groups
    }

    pub fn push_event(&mut self, event: &SectorEventType) -> SectorEvenReturnType {
        // self.event_stack.push(event.clone());
        self.handle_event(event)
    }

    fn handle_event(&mut self, event: &SectorEventType) -> SectorEvenReturnType {
        match event {
            SectorEventType::External(ExternalSectorEventType::GetSectorState) => {
                return SectorState(self.clone());
            }
            SectorEventType::Internal(InternalSectorEventType::Enter(construct_name, group_id)) => {
                Entered(
                    self.enter_sector(construct_name.clone(), group_id.clone()),
                )
            }
            SectorEventType::Internal(InternalSectorEventType::Leave(construct_name)) => {
                match self.leave_sector(construct_name) {
                    Ok(_) => Approved,
                    Err(message) => Denied(message)
                }
            }
            SectorEventType::External(ExternalSectorEventType::MoveToGroup(construct_name, group_id)) => {
                match self.move_to_group(construct_name.clone(), group_id.clone()) {
                    Ok(new_group_id) => Entered(new_group_id),
                    Err(message) => Denied(message)
                }
            }
        }
    }

    fn move_to_group(&mut self, construct_name: String, construct_target: Option<usize>) -> Result<usize, String> {
        if let Some(construct_source) = self.groups.iter()
            .position(|group| group.contains(&construct_name)) {
            if construct_target.is_some() && construct_source.eq(&construct_target.unwrap()) {
                return Ok(construct_source); //If you are traveling from and to the same group then you just stay without any change.
            }

            let result = Ok(self.add_construct_to_group(&construct_name, construct_target));
            self.remove_construct_from_group(&construct_name, construct_source);
            return result;
        } else {
            Err(format!("Construct with name {} does not seem to be a member of any group in sector {:?} so it cannot move.", construct_name, self.position))
        }
    }

    fn add_construct_to_group(&mut self, construct_name: &String, group_id: Option<usize>) -> usize {
        let mut new_address = 0;

        let mut target_exists = false;
        if let Some(address) = group_id {
            if let Some(group) = self.groups.get_mut(address) {
                group.push(construct_name.clone());
                target_exists = true;
                new_address = address;
            }
        }

        if !target_exists {
            match self.groups.iter()
                .position(|group| group.is_empty()) {
                Some(address) => {
                    self.groups.get_mut(address).unwrap().push(construct_name.clone());
                    new_address = address;
                }
                None => {
                    let mut new_group = Vec::new();
                    new_group.push(construct_name.clone());
                    self.groups.push(new_group);
                    new_address = self.groups.len() - 1;
                }
            }
        }

        new_address
    }

    fn remove_construct_from_group(&mut self, construct_name: &String, group_id: usize) {
        if let Some(group) = self.groups.get_mut(group_id) {
            group.retain(|c| c.ne(construct_name));
        }
    }

    fn leave_sector(&mut self, construct_name: &String) -> Result<(), String> {
        if let Some(construct_source) = self.groups.iter()
            .position(|group| group.contains(&construct_name)) {
            self.remove_construct_from_group(&construct_name, construct_source);
            Ok(())
        } else {
            Err(format!("Construct with name {} does not seem to be a member of any group in sector {:?} so it cannot move.", construct_name, self.position))
        }
    }

    pub fn enter_sector(&mut self, construct_name: String, group_id: Option<usize>) -> usize {
        self.add_construct_to_group(&construct_name, group_id)
    }
}

#[cfg(test)]
mod tests_int {
    use crate::sector::{ExternalSectorEventType, InternalSectorEventType, Sector, SectorEvenReturnType, SectorEventType, SectorPosition};

    #[test]
    fn one_construct() {
        let position = SectorPosition { x: 4, y: 3, z: 2 };
        let groups = Vec::new();
        let mut sector = Sector::new(groups, position);

        assert_eq!(0, sector.groups.len());
        assert_eq!(SectorEvenReturnType::Entered(0), sector.handle_event(&SectorEventType::Internal(InternalSectorEventType::Enter("construct_1".to_string(), Some(0)))));
        assert_eq!(1, sector.groups.len());
        assert_eq!(&"construct_1".to_string(), sector.groups.get(0).unwrap().get(0).unwrap());

        assert_eq!(SectorEvenReturnType::Entered(0), sector.handle_event(&SectorEventType::External(ExternalSectorEventType::MoveToGroup("construct_1".to_string(), Some(0)))));
        assert_eq!(&"construct_1".to_string(), sector.groups.get(0).unwrap().get(0).unwrap());
        assert_eq!(1, sector.groups.len());

        assert_eq!(SectorEvenReturnType::Entered(1), sector.handle_event(&SectorEventType::External(ExternalSectorEventType::MoveToGroup("construct_1".to_string(), None))));
        assert!(sector.groups.get(0).unwrap().is_empty());
        assert_eq!(&"construct_1".to_string(), sector.groups.get(1).unwrap().get(0).unwrap());
        assert_eq!(2, sector.groups.len());

        assert_eq!(SectorEvenReturnType::Entered(0), sector.handle_event(&SectorEventType::External(ExternalSectorEventType::MoveToGroup("construct_1".to_string(), Some(0)))));
        assert_eq!(&"construct_1".to_string(), sector.groups.get(0).unwrap().get(0).unwrap());
        assert!(sector.groups.get(1).unwrap().is_empty());
        assert_eq!(2, sector.groups.len());

        assert_eq!(SectorEvenReturnType::Entered(1), sector.handle_event(&SectorEventType::External(ExternalSectorEventType::MoveToGroup("construct_1".to_string(), Some(42)))));
        assert!(sector.groups.get(0).unwrap().is_empty());
        assert_eq!(&"construct_1".to_string(), sector.groups.get(1).unwrap().get(0).unwrap());
        assert_eq!(2, sector.groups.len());

        assert_eq!(SectorEvenReturnType::Approved, sector.handle_event(&SectorEventType::Internal(InternalSectorEventType::Leave("construct_1".to_string()))));
        assert!(sector.groups.get(0).unwrap().is_empty());
        assert!(sector.groups.get(1).unwrap().is_empty());
        assert_eq!(2, sector.groups.len());
    }


    #[test]
    fn multiple_construct() {
        let position = SectorPosition { x: 4, y: 3, z: 2 };
        let groups = Vec::new();
        let mut sector = Sector::new(groups, position);

        assert_eq!(0, sector.groups.len());
        assert_eq!(SectorEvenReturnType::Entered(0), sector.handle_event(&SectorEventType::Internal(InternalSectorEventType::Enter("construct_1".to_string(), None))));
        assert_eq!(1, sector.groups.len());
        assert_eq!(&"construct_1".to_string(), sector.groups.get(0).unwrap().get(0).unwrap());

        assert_eq!(SectorEvenReturnType::Entered(1), sector.handle_event(&SectorEventType::Internal(InternalSectorEventType::Enter("construct_2".to_string(), None))));
        assert_eq!(2, sector.groups.len());
        assert_eq!(&"construct_1".to_string(), sector.groups.get(0).unwrap().get(0).unwrap());
        assert_eq!(&"construct_2".to_string(), sector.groups.get(1).unwrap().get(0).unwrap());

        assert_eq!(SectorEvenReturnType::Entered(2), sector.handle_event(&SectorEventType::External(ExternalSectorEventType::MoveToGroup("construct_1".to_string(), None))));
        assert_eq!(3, sector.groups.len());
        assert!(sector.groups.get(0).unwrap().is_empty());
        assert_eq!(&"construct_2".to_string(), sector.groups.get(1).unwrap().get(0).unwrap());
        assert_eq!(&"construct_1".to_string(), sector.groups.get(2).unwrap().get(0).unwrap());

        assert_eq!(SectorEvenReturnType::Entered(2), sector.handle_event(&SectorEventType::External(ExternalSectorEventType::MoveToGroup("construct_2".to_string(), Some(2)))));
        assert_eq!(3, sector.groups.len());
        assert!(sector.groups.get(0).unwrap().is_empty());
        assert!(sector.groups.get(1).unwrap().is_empty());
        assert_eq!(&"construct_2".to_string(), sector.groups.get(2).unwrap().get(1).unwrap());
        assert_eq!(&"construct_1".to_string(), sector.groups.get(2).unwrap().get(0).unwrap());

        assert_eq!(SectorEvenReturnType::Entered(0), sector.handle_event(&SectorEventType::External(ExternalSectorEventType::MoveToGroup("construct_1".to_string(), None))));
        assert_eq!(3, sector.groups.len());
        assert_eq!(&"construct_1".to_string(), sector.groups.get(0).unwrap().get(0).unwrap());
        assert!(sector.groups.get(1).unwrap().is_empty());
        assert_eq!(&"construct_2".to_string(), sector.groups.get(2).unwrap().get(0).unwrap());

        assert_eq!(SectorEvenReturnType::Approved, sector.handle_event(&SectorEventType::Internal(InternalSectorEventType::Leave("construct_1".to_string()))));
        assert_eq!(3, sector.groups.len());
        assert!(sector.groups.get(0).unwrap().is_empty());
        assert!(sector.groups.get(1).unwrap().is_empty());
        assert_eq!(&"construct_2".to_string(), sector.groups.get(2).unwrap().get(0).unwrap());

        assert_eq!(
            Err("Construct with name construct_1 does not seem to be a member of any group in sector SectorPosition { x: 4, y: 3, z: 2 } so it cannot move.".to_string()),
            sector.leave_sector(&"construct_1".to_string())
        );

        assert_eq!(SectorEvenReturnType::Entered(0), sector.handle_event(&SectorEventType::External(ExternalSectorEventType::MoveToGroup("construct_2".to_string(), None))));
        assert_eq!(&"construct_2".to_string(), sector.groups.get(0).unwrap().get(0).unwrap());
        assert!(sector.groups.get(1).unwrap().is_empty());
        assert!(sector.groups.get(2).unwrap().is_empty());
    }
}

