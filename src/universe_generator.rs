use std::collections::HashMap;
use std::fs;
use std::ops::Add;

use serde::{Deserialize, Serialize};

pub use crate::construct::amount::Amount;
use crate::construct::construct::{Construct, ConstructEventType, ExternalConstructEventType};
use crate::construct::production_module::ProductionModule;
use crate::construct_module::ConstructModuleType::Production as ProductionModuleType;
use crate::MainConfig;
use crate::my_little_universe::MyLittleUniverse;
use crate::products::Product;
use crate::time::TimeStackState;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct UniverseGeneratorConfig {
    method: String,
}

pub fn generate_universe(config: &MainConfig) -> MyLittleUniverse {
    let generator_config = read_universe_generator_config_file(&config.config_name().to_string());

    if generator_config.method.eq("generate_performance_test_universe") {
        generate_performance_test_universe(config.universe_name.clone())
    } else if generator_config.method.eq("generate_simple_universe") {
        generate_simple_universe(config.universe_name.clone())
    } else {
        panic!("Does not know the generate universe method, got {}", generator_config.method)
    }
}


fn read_universe_generator_config_file(config_name: &String) -> UniverseGeneratorConfig {
    let config_folder = "./config/".to_string().add(config_name);

    let universe_generator_config_path = config_folder.to_string().add("/universe_generation.json");
    println!("Using universe_generator config main_config_path: {}", universe_generator_config_path);

    let universe_generator_setup_config = fs::read_to_string(universe_generator_config_path)
        .expect("Something went wrong reading the file");

    let niverse_generator_config: UniverseGeneratorConfig = serde_json::from_str(universe_generator_setup_config.as_str())
        .expect("Something went wrong parsing the file from");

    niverse_generator_config
}

pub fn generate_simple_universe(universe_name: String) -> MyLittleUniverse {
    let mut construct = Construct::new("The_base".to_string(), 500);
    let ore_production = ProductionModule::new(
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

    for i in 1..999999 {
        let mut construct = Construct::new(format!("{}{}", i, "The_base".to_string()), 500);
        let ore_production = ProductionModule::new(
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