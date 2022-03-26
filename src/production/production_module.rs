use std::cmp;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::construct_module::{CanHandleNextTurn, ConstructModuleType};
use crate::production::cosntruct;
use crate::production::cosntruct::Construct;
use crate::products::Product;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ProductionModule {
    name: String,
    input: Vec<Amount>,
    output: Vec<Amount>,
    production_time: u32,
    production_trigger_time: u64,
    stored_input: bool,
    stored_output: bool
}

impl ProductionModule {
    fn have_all_inputs(&mut self, construct: &Construct) -> bool {
        println!("construct in have_all_input {:?}", construct);
        for input in &self.input {
            match construct.current_storage().get(input.product()) {
                Some(amount_stored) => {
                    if input.amount > *amount_stored {
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

    fn subtract_all_inputs(&mut self, construct: &mut Construct) {
        for input in &self.input {
            let leftover = construct.unload_request(&Amount::new(input.product().clone(), input.amount()));

            if leftover != input.amount() {
                panic!("Concurrency issue: subtract_all_inputs should be called right after have_all_inputs and ensure room")
            }
        }
    }

    fn have_room_for_outputs(&mut self, construct: &Construct) -> bool {
        let mut total_need = 0;
        println!("production in have_room_for_outputs {:?}", self);
        for output in &self.output {
            total_need += output.amount;
        }
        return total_need + construct.current_storage().values().sum::<u32>() <= construct.capacity();
    }

    pub fn will_output(&self, current_turn: &u64) -> Option<&Vec<Amount>> {
        if dbg!(self.production_trigger_time > 0) && dbg!(current_turn >= &self.production_trigger_time) {
            return Some(self.output())
        }
        None
    }

    pub fn require_input(&self, current_turn: &u64) -> Option<&Vec<Amount>> {
        if current_turn >= &self.production_trigger_time {
            return Some(self.input())
        }
        None
    }

    pub fn handle_turn(&mut self, current_turn: &u64) {
        if self.production_trigger_time <= *current_turn {
            if self.stored_input && !self.stored_output {
                self.production_trigger_time = current_turn + u64::from(self.production_time);
                self.stored_input = false;
            }
        }
    }

    fn add_all_outputs(&mut self, construct: &mut Construct) {
        for mut output in &mut self.output {
            let leftover = construct.load_request(&Amount::new(output.product().clone(), output.amount));

            if leftover != 0 {
                panic!("Concurrency issue: add_all_outputs should be called right after have_room_for_outputs and ensure room")
            }
        }
    }

    pub fn new(name: String, input: Vec<Amount>, output: Vec<Amount>, production_time: u32, production_trigger_time: u64) -> Self {
        Self { name, input, output, production_time, production_trigger_time, stored_input: false, stored_output: false }
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

    pub fn stored_input(&self) -> bool {
        self.stored_input
    }
    pub fn stored_output(&self) -> bool {
        self.stored_output
    }

    pub fn set_stored_input(&mut self, stored_input: bool) {
        self.stored_input = stored_input;
    }
    pub fn set_stored_output(&mut self, stored_output: bool) {
        self.stored_output = stored_output;
    }

    pub fn next_turn(&mut self, current_turn: &u64, construct: &mut Construct) {
        if current_turn >= &self.production_trigger_time {
            if self.production_trigger_time > 0 && self.have_room_for_outputs(&construct) {
                println!("construct in have_room_for_outputs {:?}", construct);
                self.add_all_outputs(construct);
            }
            if self.have_all_inputs(&construct) {
                self.subtract_all_inputs(construct);
                self.production_trigger_time = current_turn + self.production_time as u64;
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Amount {
    product: Product,
    amount: u32,
}

impl Amount {
    pub fn new(product: Product, amount: u32) -> Self {
        Self { product, amount }
    }

    pub fn product(&self) -> &Product {
        &self.product
    }
    pub fn amount(&self) -> u32 {
        self.amount
    }
}


#[cfg(test)]
mod tests_int {
    use std::collections::HashMap;

    use crate::construct_module::CanHandleNextTurn;
    use crate::production::cosntruct::Construct;
    use crate::production::production_module::{Amount, ProductionModule};
    use crate::products::Product;

    #[test]
    fn it_works() {
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

        construct.load_request(&Amount::new(Product::PowerCells, 200));

        ore_production.next_turn(&1, &mut construct);
        metal_production.next_turn(&1, &mut construct);

        assert_eq!(2, ore_production.production_trigger_time);
        assert_eq!(0, metal_production.production_trigger_time);

        assert_eq!(199, *construct.current_storage().get(&Product::PowerCells).unwrap());
        assert_eq!(None, construct.current_storage().get(&Product::Ores));
        assert_eq!(None, construct.current_storage().get(&Product::Metals));

        ore_production.next_turn(&2, &mut construct);
        metal_production.next_turn(&2, &mut construct);

        assert_eq!(3, ore_production.production_trigger_time);
        assert_eq!(0, metal_production.production_trigger_time);

        assert_eq!(198, *construct.current_storage().get(&Product::PowerCells).unwrap());
        assert_eq!(2, *construct.current_storage().get(&Product::Ores).unwrap());
        assert_eq!(None, construct.current_storage().get(&Product::Metals));

        ore_production.next_turn(&3, &mut construct);
        metal_production.next_turn(&3, &mut construct);

        assert_eq!(4, ore_production.production_trigger_time);
        assert_eq!(6, metal_production.production_trigger_time);

        assert_eq!(195, *construct.current_storage().get(&Product::PowerCells).unwrap());
        assert_eq!(None, construct.current_storage().get(&Product::Ores));
        assert_eq!(None, construct.current_storage().get(&Product::Metals));

        ore_production.next_turn(&4, &mut construct);
        metal_production.next_turn(&4, &mut construct);

        assert_eq!(5, ore_production.production_trigger_time);
        assert_eq!(6, metal_production.production_trigger_time);

        assert_eq!(194, *construct.current_storage().get(&Product::PowerCells).unwrap());
        assert_eq!(2, *construct.current_storage().get(&Product::Ores).unwrap());
        assert_eq!(None, construct.current_storage().get(&Product::Metals));


        ore_production.next_turn(&5, &mut construct);
        metal_production.next_turn(&5, &mut construct);

        assert_eq!(6, ore_production.production_trigger_time);
        assert_eq!(6, metal_production.production_trigger_time);

        assert_eq!(193, *construct.current_storage().get(&Product::PowerCells).unwrap());
        assert_eq!(4, *construct.current_storage().get(&Product::Ores).unwrap());
        assert_eq!(None, construct.current_storage().get(&Product::Metals));

        ore_production.next_turn(&6, &mut construct);
        metal_production.next_turn(&6, &mut construct);

        assert_eq!(7, ore_production.production_trigger_time);
        assert_eq!(9, metal_production.production_trigger_time);

        assert_eq!(190, *construct.current_storage().get(&Product::PowerCells).unwrap());
        assert_eq!(2, *construct.current_storage().get(&Product::Ores).unwrap());
        assert_eq!(1, *construct.current_storage().get(&Product::Metals).unwrap());
    }
}