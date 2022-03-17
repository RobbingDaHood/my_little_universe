use crate::construct_module::{CanHandleLoadingRequests, CanHandleNextTurn};
use crate::products::Product;
use serde::{Deserialize, Serialize};
use crate::station::LoadingRequest;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SingleProductStorageModule {
    name: String,
    product: Product,
    capacity: u32,
    current_storage: u32,
}

impl SingleProductStorageModule {
    pub fn new(name: String, product: Product, capacity: u32, current_storage: u32) -> Self {
        Self { name, product, capacity, current_storage }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn product(&self) -> &Product {
        &self.product
    }
    pub fn capacity(&self) -> u32 {
        self.capacity
    }
    pub fn current_storage(&self) -> u32 {
        self.current_storage
    }

    //TODO Can I make this more private and is it concurrency safe?
    pub fn increase_current_storage(&mut self, change: u32) {
        self.current_storage += change;
    }

    //TODO Can I make this more private and is it concurrency safe?
    pub fn reduce_current_storage(&mut self, change: u32) {
        self.current_storage -= change;
    }
}