use serde::{Deserialize, Serialize};
use crate::construct_module::ConstructModuleType;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Eq, Hash)]
pub enum Product {
    Ores,
    Metals,
    PowerCells,
    Module(ConstructModuleType)
}