use serde::{Deserialize, Serialize};

use crate::construct::production_module::ProductionModule;

pub trait CanHandleNextTurn {
    fn next_turn(&mut self, current_turn: &u64);
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Eq, Hash)]
pub enum ConstructModuleType {
    Production(ProductionModule)
}

impl ConstructModuleType {
    pub fn name(&self) -> &str {
        return match self {
            ConstructModuleType::Production(production_module) => {
                production_module.name()
            }
        };
    }
}

