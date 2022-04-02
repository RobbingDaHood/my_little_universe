use std::collections::HashMap;

pub use crate::construct::amount::Amount;
use crate::construct::construct::{Construct, ConstructEventType, ExternalConstructEventType};
use crate::construct::production_module::ProductionModule;
use crate::construct_module::ConstructModuleType::Production as ProductionModuleType;
use crate::my_little_universe::MyLittleUniverse;
use crate::products::Product;
use crate::time::TimeStackState;

pub fn generate_simple_universe(universe_name: String) -> MyLittleUniverse {
    let mut construct = Construct::new("The_base".to_string(), 500);
    let mut ore_production = ProductionModule::new(
        "PowerToOre".to_string(),
        vec![Amount::new(Product::PowerCells, 1)],
        vec![Amount::new(Product::Ores, 2)],
        1,
        0,
    );
    assert_eq!(Ok(()), construct.install(ProductionModuleType(ore_production.clone())));

    let mut constructs: HashMap<String, Construct> = HashMap::new();
    constructs.insert(construct.name().to_string(), construct);

    MyLittleUniverse::new(universe_name, TimeStackState::new(), constructs)
}


pub fn generate_performance_test_universe(universe_name: String) -> MyLittleUniverse {
    let mut constructs: HashMap<String, Construct> = HashMap::new();

    for i in 1..9999999 {
        let mut construct = Construct::new(format!("{}{}", i, "The_base".to_string()), 500);
        let mut ore_production = ProductionModule::new(
            "PowerToOre".to_string(),
            vec![Amount::new(Product::PowerCells, 1)],
            vec![Amount::new(Product::Ores, 2)],
            1,
            0,
        );
        assert_eq!(Ok(()), construct.install(ProductionModuleType(ore_production.clone())));

        construct.push_event(&ConstructEventType::External(ExternalConstructEventType::RequestLoad(Amount::new(Product::PowerCells, 200))));

        constructs.insert(construct.name().to_string(), construct);
    }

    MyLittleUniverse::new(universe_name, TimeStackState::new(), constructs)
}