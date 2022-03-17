use std::collections::HashMap;
use crate::construct_module::CanHandleNextTurn;
use crate::production::single_product_storage::SingleProductStorageModule;
use crate::products::Product;
use serde::{Deserialize, Serialize};

pub struct ProductionModule {
    name: String,
    input: Vec<Amount>,
    output: Vec<Amount>,
    production_time: u32,
    production_trigger_time: u64,
}

impl ProductionModule {
    fn have_all_inputs(&mut self, possible_storage: &HashMap<String, SingleProductStorageModule>) -> bool {
        for input in &self.input {
            match possible_storage.get(input.connected_storage_name()) {
                Some(connected_storage) => {
                    if input.amount > connected_storage.current_storage() {
                        return false;
                    }
                }
                None => {
                    return false;
                }
            }
        }
        return true;
    }

    fn subtract_all_inputs(&mut self, possible_storage: &mut HashMap<String, SingleProductStorageModule>) {
        for mut input in &mut self.input {
            match possible_storage.get_mut(input.connected_storage_name()) {
                Some(connected_storage) => {
                    connected_storage.reduce_current_storage(input.amount);
                }
                None => {
                    panic!("Tried to reduce_current_storage on a non existing storage")
                }
            }
        }
    }

    fn have_room_for_outputs(&mut self, possible_storage: &HashMap<String, SingleProductStorageModule>) -> bool {
        for output in &self.output {
           match possible_storage.get(output.connected_storage_name()) {
               Some(connected_storage) => {
                   if connected_storage.current_storage() + output.amount > connected_storage.capacity() {
                       return false;
                   }
               }
               None => {
                   return false;
               }
           }
        }
        return true;
    }

    fn add_all_outputs(&mut self, possible_storage: &mut HashMap<String, SingleProductStorageModule>) {
        for mut output in &mut self.output {
            match possible_storage.get_mut(output.connected_storage_name()) {
                Some(connected_storage) => {
                    connected_storage.increase_current_storage(output.amount);
                }
                None => {
                    panic!("Tried to increase_current_storage that does not exist")
                }
            }
        }
    }
    pub fn new(name: String, input: Vec<Amount>, output: Vec<Amount>, production_time: u32, production_trigger_time: u64) -> Self {
        Self { name, input, output, production_time, production_trigger_time }
    }


    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn input(&self) -> &Vec<Amount> {
        &self.input
    }
    pub fn output(&self) -> &Vec<Amount> {
        &self.output
    }
    pub fn production_time(&self) -> u32 {
        self.production_time
    }
    pub fn production_trigger_time(&self) -> u64 {
        self.production_trigger_time
    }

    fn next_turn(&mut self, current_turn: &u64, possible_storage: &mut HashMap<String, SingleProductStorageModule>) {
        if current_turn >= &self.production_trigger_time {
            if self.production_trigger_time > 0 && self.have_room_for_outputs(&possible_storage) {
                self.add_all_outputs(possible_storage);
            }
            if self.have_all_inputs(&possible_storage) {
                self.subtract_all_inputs(possible_storage);
                self.production_trigger_time = current_turn + self.production_time as u64;
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Amount {
    product: Product,
    amount: u32,
    connected_storage_name: String,
}

impl Amount {
    pub fn new(product: Product, amount: u32, connected_storage_name: String) -> Self {
        Self { product, amount, connected_storage_name }
    }

    pub fn product(&self) -> &Product {
        &self.product
    }
    pub fn amount(&self) -> u32 {
        self.amount
    }
    pub fn connected_storage_name(&self) -> &str {
        &self.connected_storage_name
    }
}


#[cfg(test)]
mod tests_int {
    use std::collections::HashMap;
    use crate::construct_module::CanHandleNextTurn;
    use crate::production::production_module::{Amount, ProductionModule};
    use crate::production::single_product_storage::SingleProductStorageModule;
    use crate::products::Product;

    #[test]
    fn it_works() {
        let ore_storage_name = "OreStorage";
        let ore_storage = SingleProductStorageModule::new(ore_storage_name.to_string(), Product::Ores, 10000, 0);
        let power_storage_name = "PowerStorage";
        let mut power_storage = SingleProductStorageModule::new(power_storage_name.to_string(), Product::PowerCells, 20000, 0);
        let metal_storage_name = "MetalStorage";
        let metal_storage = SingleProductStorageModule::new(metal_storage_name.to_string(), Product::Metals, 20000, 0);

        let mut all_storage: HashMap<String, SingleProductStorageModule> = HashMap::new();
        all_storage.insert(ore_storage_name.to_string(), ore_storage);
        all_storage.insert(power_storage_name.to_string(), power_storage);
        all_storage.insert(metal_storage_name.to_string(), metal_storage);

        let output_ore = Amount::new(Product::Ores, 2, ore_storage_name.to_string());
        let input_ore = Amount::new(Product::Ores, 4, ore_storage_name.to_string());
        let input_power = Amount::new(Product::PowerCells, 1, power_storage_name.to_string());
        let input_power_2 = Amount::new(Product::PowerCells, 2, power_storage_name.to_string());
        let output_metals = Amount::new(Product::Metals, 1, metal_storage_name.to_string());

        let mut ore_production = ProductionModule::new(
            "PowerToOre".to_string(),
            vec![input_power],
            vec![output_ore],
            1,
            0,
        );
        let mut metal_production = ProductionModule::new(
            "PowerToOre".to_string(),
            vec![input_power_2, input_ore],
            vec![output_metals],
            3,
            0,
        );

        all_storage.get_mut(power_storage_name).unwrap().increase_current_storage(1000);

        ore_production.next_turn(&1, &mut all_storage);
        metal_production.next_turn(&1, &mut all_storage);

        assert_eq!(2, ore_production.production_trigger_time);
        assert_eq!(0, metal_production.production_trigger_time);

        assert_eq!(999, all_storage.get(power_storage_name).unwrap().current_storage());
        assert_eq!(0, all_storage.get(ore_storage_name).unwrap().current_storage());
        assert_eq!(0, all_storage.get(metal_storage_name).unwrap().current_storage());

        ore_production.next_turn(&2, &mut all_storage);
        metal_production.next_turn(&2, &mut all_storage);

        assert_eq!(3, ore_production.production_trigger_time);
        assert_eq!(0, metal_production.production_trigger_time);

        assert_eq!(998, all_storage.get(power_storage_name).unwrap().current_storage());
        assert_eq!(2, all_storage.get(ore_storage_name).unwrap().current_storage());
        assert_eq!(0, all_storage.get(metal_storage_name).unwrap().current_storage());

        ore_production.next_turn(&3, &mut all_storage);
        metal_production.next_turn(&3, &mut all_storage);

        assert_eq!(4, ore_production.production_trigger_time);
        assert_eq!(6, metal_production.production_trigger_time);

        assert_eq!(995, all_storage.get(power_storage_name).unwrap().current_storage());
        assert_eq!(0, all_storage.get(ore_storage_name).unwrap().current_storage());
        assert_eq!(0, all_storage.get(metal_storage_name).unwrap().current_storage());

        ore_production.next_turn(&4, &mut all_storage);
        metal_production.next_turn(&4, &mut all_storage);

        assert_eq!(5, ore_production.production_trigger_time);
        assert_eq!(6, metal_production.production_trigger_time);

        assert_eq!(994, all_storage.get(power_storage_name).unwrap().current_storage());
        assert_eq!(2, all_storage.get(ore_storage_name).unwrap().current_storage());
        assert_eq!(0, all_storage.get(metal_storage_name).unwrap().current_storage());

        ore_production.next_turn(&5, &mut all_storage);
        metal_production.next_turn(&5, &mut all_storage);

        assert_eq!(6, ore_production.production_trigger_time);
        assert_eq!(6, metal_production.production_trigger_time);

        assert_eq!(993, all_storage.get(power_storage_name).unwrap().current_storage());
        assert_eq!(4, all_storage.get(ore_storage_name).unwrap().current_storage());
        assert_eq!(0, all_storage.get(metal_storage_name).unwrap().current_storage());

        ore_production.next_turn(&6, &mut all_storage);
        metal_production.next_turn(&6, &mut all_storage);

        assert_eq!(7, ore_production.production_trigger_time);
        assert_eq!(9, metal_production.production_trigger_time);

        assert_eq!(990, all_storage.get(power_storage_name).unwrap().current_storage());
        assert_eq!(2, all_storage.get(ore_storage_name).unwrap().current_storage());
        assert_eq!(1, all_storage.get(metal_storage_name).unwrap().current_storage());
    }
}