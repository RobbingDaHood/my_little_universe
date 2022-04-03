use serde::{Deserialize, Serialize};

use crate::construct::construct_position::ConstructPosition::{Docked, Nowhere};

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

pub fn handle_event(event: &ExternalConstructPositionEventType) -> ConstructPosition {
    match event {
        ExternalConstructPositionEventType::Dock(construct_name) => {
            Docked(construct_name.clone())
        }
        ExternalConstructPositionEventType::Undock => {
            Nowhere
        }
    }
}

