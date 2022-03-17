use std::fmt::format;
use crate::products::Product;
use serde::{Deserialize, Serialize};

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

