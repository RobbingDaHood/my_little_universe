use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::construct_module::ConstructModuleType;
use crate::products::Product;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Construct {
    name: String,
    capacity: u32,
    current_storage: HashMap<Product, u32>,
    modules: Vec<ConstructModuleType>,
}

impl Construct {
    pub fn new(name: String, capacity: u32) -> Self {
        Construct { name, capacity, current_storage: HashMap::new(), modules: Vec::new() }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn capacity(&self) -> u32 {
        self.capacity
    }
    pub fn current_storage(&self) -> &HashMap<Product, u32> {
        &self.current_storage
    }

    pub fn unload(&mut self, product: &Product, amount: u32) -> Result<(), String> {
        match self.current_storage.get_mut(product) {
            Some(amount_stored) => {
                if *amount_stored > amount {
                    *amount_stored -= amount;
                    Ok(())
                } else if *amount_stored == amount {
                    return match self.current_storage.remove(product) {
                        Some(_) => { Ok(()) }
                        None => { panic!("Just checked that product were there, but now it is not. Concurrency issue. Good luck! ") }
                    }
                } else {
                    Err(format!("There is not {} {:?} in storage. Only had {}", amount, product, amount_stored))
                }
            }
            None => {
                Err(format!("Tried to reduce_current_storage on a non existing storage, {:?}", product))
            }
        }
    }

    pub fn load(&mut self, product: &Product, amount: u32) -> Result<(), String> {
        if self.capacity >= self.current_storage.values().sum::<u32>() + amount {
            match self.current_storage.get_mut(product) {
                Some(amount_stored) => {
                    *amount_stored += amount;
                    println!("some");
                }
                None => {
                    self.current_storage.insert(product.clone(), amount);
                    println!("None");
                }
            }
            Ok(())
        } else {
            Err(format!("Not enough capacity to load {}", amount))
        }
    }
}


