use std::collections::HashMap;
use std::fmt::format;

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Sector {
    groups: Vec<Vec<String>>,
    //index is group id
    position: SectorPosition,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SectorPosition {
    x: u8,
    y: u8,
    z: u8,
}

impl Sector {
    pub fn new(groups: Vec<Vec<String>>, position: SectorPosition) -> Self {
        Sector { groups, position }
    }

    pub fn groups(&self) -> &Vec<Vec<String>> {
        &self.groups
    }
    pub fn position(&self) -> &SectorPosition {
        &self.position
    }

    pub fn move_to_group(&mut self, construct_name: String, construct_target: Option<usize>) -> Result<usize, String> {
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

    fn add_construct_to_group(&mut self, construct_name: &String, construct_target: Option<usize>) -> usize {
        let mut new_address = 0;

        let mut target_exists = false;
        if let Some(address) = construct_target {
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

    fn remove_construct_from_group(&mut self, construct_name: &String, address: usize) {
        if let Some(group) = self.groups.get_mut(address) {
            group.retain(|c| c.ne(construct_name));
        }
    }

    pub fn leave_sector(&mut self, construct_name: &String) -> Result<(), String> {
        if let Some(construct_source) = self.groups.iter()
            .position(|group| group.contains(&construct_name)) {
            self.remove_construct_from_group(&construct_name, construct_source);
            Ok(())
        } else {
            Err(format!("Construct with name {} does not seem to be a member of any group in sector {:?} so it cannot move.", construct_name, self.position))
        }
    }

    pub fn enter_sector(&mut self, construct_name: String, construct_target: Option<usize>) -> usize {
        self.add_construct_to_group(&construct_name, construct_target)
    }
}

#[cfg(test)]
mod tests_int {
    use crate::sector::{Sector, SectorPosition};

    #[test]
    fn one_construct() {
        let position = SectorPosition { x: 4, y: 3, z: 2 };
        let groups = Vec::new();
        let mut sector = Sector::new(groups, position);

        assert_eq!(0, sector.groups.len());
        assert_eq!(0, sector.enter_sector("construct_1".to_string(), Some(0)));
        assert_eq!(1, sector.groups.len());
        assert_eq!(&"construct_1".to_string(), sector.groups.get(0).unwrap().get(0).unwrap());

        assert_eq!(Ok(0), sector.move_to_group("construct_1".to_string(), Some(0)));
        assert_eq!(&"construct_1".to_string(), sector.groups.get(0).unwrap().get(0).unwrap());
        assert_eq!(1, sector.groups.len());

        assert_eq!(Ok(1), sector.move_to_group("construct_1".to_string(), None));
        assert!(sector.groups.get(0).unwrap().is_empty());
        assert_eq!(&"construct_1".to_string(), sector.groups.get(1).unwrap().get(0).unwrap());
        assert_eq!(2, sector.groups.len());

        assert_eq!(Ok(0), sector.move_to_group("construct_1".to_string(), Some(0)));
        assert_eq!(&"construct_1".to_string(), sector.groups.get(0).unwrap().get(0).unwrap());
        assert!(sector.groups.get(1).unwrap().is_empty());
        assert_eq!(2, sector.groups.len());

        assert_eq!(Ok(1), sector.move_to_group("construct_1".to_string(), Some(42)));
        assert!(sector.groups.get(0).unwrap().is_empty());
        assert_eq!(&"construct_1".to_string(), sector.groups.get(1).unwrap().get(0).unwrap());
        assert_eq!(2, sector.groups.len());

        sector.leave_sector(&"construct_1".to_string());
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
        assert_eq!(0, sector.enter_sector("construct_1".to_string(), None));
        assert_eq!(1, sector.groups.len());
        assert_eq!(&"construct_1".to_string(), sector.groups.get(0).unwrap().get(0).unwrap());

        assert_eq!(1, sector.enter_sector("construct_2".to_string(), None));
        assert_eq!(2, sector.groups.len());
        assert_eq!(&"construct_1".to_string(), sector.groups.get(0).unwrap().get(0).unwrap());
        assert_eq!(&"construct_2".to_string(), sector.groups.get(1).unwrap().get(0).unwrap());

        assert_eq!(Ok(2), sector.move_to_group("construct_1".to_string(), None));
        assert_eq!(3, sector.groups.len());
        assert!(sector.groups.get(0).unwrap().is_empty());
        assert_eq!(&"construct_2".to_string(), sector.groups.get(1).unwrap().get(0).unwrap());
        assert_eq!(&"construct_1".to_string(), sector.groups.get(2).unwrap().get(0).unwrap());

        assert_eq!(Ok(2), sector.move_to_group("construct_2".to_string(), Some(2)));
        assert_eq!(3, sector.groups.len());
        assert!(sector.groups.get(0).unwrap().is_empty());
        assert!(sector.groups.get(1).unwrap().is_empty());
        assert_eq!(&"construct_2".to_string(), sector.groups.get(2).unwrap().get(1).unwrap());
        assert_eq!(&"construct_1".to_string(), sector.groups.get(2).unwrap().get(0).unwrap());

        assert_eq!(Ok(0), sector.move_to_group("construct_1".to_string(), None));
        assert_eq!(3, sector.groups.len());
        assert_eq!(&"construct_1".to_string(), sector.groups.get(0).unwrap().get(0).unwrap());
        assert!(sector.groups.get(1).unwrap().is_empty());
        assert_eq!(&"construct_2".to_string(), sector.groups.get(2).unwrap().get(0).unwrap());

        assert_eq!(Ok(()), sector.leave_sector(&"construct_1".to_string()));
        assert_eq!(3, sector.groups.len());
        assert!(sector.groups.get(0).unwrap().is_empty());
        assert!(sector.groups.get(1).unwrap().is_empty());
        assert_eq!(&"construct_2".to_string(), sector.groups.get(2).unwrap().get(0).unwrap());

        assert_eq!(
            Err("Construct with name construct_1 does not seem to be a member of any group in sector SectorPosition { x: 4, y: 3, z: 2 } so it cannot move.".to_string()),
            sector.leave_sector(&"construct_1".to_string())
        );

        assert_eq!(Ok(0), sector.move_to_group("construct_2".to_string(), None));
        assert_eq!(&"construct_2".to_string(), sector.groups.get(0).unwrap().get(0).unwrap());
        assert!(sector.groups.get(1).unwrap().is_empty());
        assert!(sector.groups.get(2).unwrap().is_empty());
    }
}

