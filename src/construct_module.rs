use std::fmt::format;
use crate::products::Product;
use serde::{Deserialize, Serialize};
use crate::construct::production_module::ProductionModule;

pub trait CanHandleNextTurn {
    fn next_turn(&mut self, current_turn: &u64);
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct LoadingRequest {
    product: Product,
    amount: u32,
}

pub trait CanHandleLoadingRequests {
    fn unload(&mut self, loading_request: LoadingRequest) -> Result<String, String>;
    fn load(&mut self, loading_request: LoadingRequest) -> Result<String, String>;
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ConstructModuleType {
    Production(ProductionModule)
}

impl ConstructModuleType {
    pub fn name(&self) -> &str {
        return match self {
            ConstructModuleType::Production(production_module) => {
                production_module.name()
            }
        }
    }
}

