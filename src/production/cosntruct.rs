use std::cmp::min;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::construct_module::ConstructModuleType;
use crate::products::Product;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Construct {
    name: String,
    capacity: u32,
    current_storage: HashMap<Product, u32>,
    modules: HashMap<String, ConstructModuleType>,
}

impl Construct {
    pub fn new(name: String, capacity: u32) -> Self {
        Construct { name, capacity, current_storage: HashMap::new(), modules: HashMap::new() }
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

    pub fn unload(&mut self, product: &Product, amount: u32) -> u32 {
        match self.current_storage.get_mut(product) {
            Some(amount_stored) => {
                if *amount_stored > amount {
                    *amount_stored -= amount;
                    amount
                } else {
                    return match self.current_storage.remove(product) {
                        Some(amount_stored) => { min(amount_stored, amount) }
                        None => { panic!("Just checked that product were there, but now it is not. Concurrency issue. Good luck! ") }
                    };
                }
            }
            None => {
                0
            }
        }
    }

    pub fn load(&mut self, product: &Product, amount: u32) -> u32 {
        let leftover_capacity = self.capacity - self.current_storage.values().sum::<u32>();
        let amount_to_be_stored = dbg!(min(leftover_capacity, amount));

        if amount_to_be_stored == 0 {
            return amount;
        }

        match self.current_storage.get_mut(product) {
            Some(amount_stored) => {
                *amount_stored += amount_to_be_stored;
                println!("some");
            }
            None => {
                self.current_storage.insert(product.clone(), amount_to_be_stored);
                println!("None");
            }
        }

        if amount_to_be_stored >= amount {
            0
        } else {
            amount - amount_to_be_stored
        }
    }

    pub fn install(&mut self, module: ConstructModuleType) -> Result<(), String> {
        let module_name = match &module {
            ConstructModuleType::Production(module) => {
                module.name()
            }
        };

        if self.modules.contains_key(module_name) {
            return Err("Module with that name already exists.".to_string())
        }

        self.modules.insert(module_name.to_string(), module);
        Ok(())
    }

    pub fn uninstall(&mut self, module_name: &String) -> Option<ConstructModuleType> {
        self.modules.remove(module_name)
    }
}


#[cfg(test)]
mod tests_int {
    use crate::construct_module::ConstructModuleType::Production;
    use crate::production::cosntruct::Construct;
    use crate::production::production_module::{Amount, ProductionModule};
    use crate::products::Product;

    #[test]
    fn load_and_unload_tries_its_best() {
        let mut construct = Construct::new("The base".to_string(), 500);
        assert_eq!(None, construct.current_storage.get(&Product::PowerCells));

        assert_eq!(200, construct.load(&Product::PowerCells, 700));
        assert_eq!(500, *construct.current_storage.get(&Product::PowerCells).unwrap());

        assert_eq!(500, construct.unload(&Product::PowerCells, 700));
        assert_eq!(None, construct.current_storage.get(&Product::PowerCells));

        assert_eq!(200, construct.load(&Product::PowerCells, 700));
        assert_eq!(500, *construct.current_storage.get(&Product::PowerCells).unwrap());

        assert_eq!(700, construct.load(&Product::PowerCells, 700));
        assert_eq!(500, *construct.current_storage.get(&Product::PowerCells).unwrap());

        assert_eq!(500, construct.unload(&Product::PowerCells, 700));
        assert_eq!(None, construct.current_storage.get(&Product::PowerCells));

        assert_eq!(0, construct.unload(&Product::PowerCells, 700));
        assert_eq!(None, construct.current_storage.get(&Product::PowerCells));
    }


    #[test]
    fn install_and_uninstall_tries_its_best() {
        let mut construct = Construct::new("The base".to_string(), 500);
        let mut ore_production = ProductionModule::new(
            "PowerToOre".to_string(),
            vec![Amount::new(Product::PowerCells, 1)],
            vec![Amount::new(Product::Ores, 2)],
            1,
            0,
        );
        let mut metal_production = ProductionModule::new(
            "OreAndEnergyToMetal".to_string(),
            vec![Amount::new(Product::PowerCells, 2), Amount::new(Product::Ores, 4)],
            vec![Amount::new(Product::Metals, 1)],
            3,
            0,
        );

        assert_eq!(Ok(()), construct.install(Production(ore_production.clone())));
        assert_eq!(Ok(()), construct.install(Production(metal_production.clone())));

        assert_eq!(Some(&Production(ore_production.clone())), construct.modules.get(ore_production.name()));
        assert_eq!(Some(&Production(metal_production.clone())), construct.modules.get(metal_production.name()));

        assert_eq!(Err("Module with that name already exists.".to_string()), construct.install(Production(ore_production.clone())));

        assert_eq!(Some(Production(ore_production.clone())), construct.uninstall(&ore_production.name().to_string()));
        assert_eq!(None, construct.uninstall(&ore_production.name().to_string()));

        assert_eq!(None, construct.modules.get(ore_production.name()));
        assert_eq!(Some(&Production(metal_production.clone())), construct.modules.get(metal_production.name()));
    }
}