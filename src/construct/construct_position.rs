use serde::{Deserialize, Serialize};

use crate::construct::construct_position::ConstructPosition::{Docked, Nowhere};
use crate::construct::construct_position::ConstructPositionEventReturnType::{ConstructCannotDockAtItself, RequestProcessed};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ConstructPosition {
    Docked(String),
    Nowhere,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ExternalConstructPositionEventType {
    Dock(String),
    Undock,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ConstructPositionEventReturnType {
    RequestProcessed,
    ConstructCannotDockAtItself,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ConstructPositionState {
    position: ConstructPosition,
    source_construct_name : String,
}

impl ConstructPositionState {
    pub fn new(position: ConstructPosition, source_construct_name: String) -> Self {
        ConstructPositionState { position, source_construct_name }
    }
    pub fn position(&self) -> &ConstructPosition {
        &self.position
    }


    pub fn handle_event(&mut self, event: &ExternalConstructPositionEventType) -> ConstructPositionEventReturnType {
        match event {
            ExternalConstructPositionEventType::Dock(construct_name) => {
                if self.source_construct_name.eq(construct_name) {
                    return ConstructCannotDockAtItself
                }
                self.position = Docked(construct_name.clone());
                RequestProcessed
            }
            ExternalConstructPositionEventType::Undock => {
                self.position = Nowhere;
                RequestProcessed
            }
        }
    }
}



