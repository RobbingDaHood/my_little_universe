use std::collections::HashMap;

pub use crate::construct::amount::Amount as ConstructAmount;
use crate::construct::construct::Construct;
use crate::construct::production_module::ProductionModule;
use crate::construct_module::ConstructModuleType::Production as ProductionModuleType;
use crate::my_little_universe::MyLittleUniverse;
use crate::products::Product;
use crate::station::{Amount, Production, Station};
use crate::station::StationEvenReturnType::StationState;
use crate::time::TimeStackState;

pub fn generate_simple_universe(universe_name: String) -> MyLittleUniverse {
    let station = Station::new("simple_station".to_string(), Production::new(
        vec![Amount::new(
            Product::PowerCells,
            1,
            0,
            10000,
        )],
        vec![Amount::new(
            Product::Ores,
            2,
            0,
            20000,
        )],
        1,
        0,
    ));

    let mut stations: HashMap<String, Station> = HashMap::new();
    stations.insert(station.name().to_string(), station);

    let mut construct = Construct::new("The_base".to_string(), 500);
    let mut ore_production = ProductionModule::new(
        "PowerToOre".to_string(),
        vec![ConstructAmount::new(Product::PowerCells, 1)],
        vec![ConstructAmount::new(Product::Ores, 2)],
        1,
        0,
    );
    assert_eq!(Ok(()), construct.install(ProductionModuleType(ore_production.clone())));

    let mut constructs: HashMap<String, Construct> = HashMap::new();
    constructs.insert(construct.name().to_string(), construct);

    MyLittleUniverse::new(universe_name, TimeStackState::new(), stations, constructs)
}


pub fn generate_performance_test_universe(universe_name: String) -> MyLittleUniverse {
    let mut stations: HashMap<String, Station> = HashMap::new();

    for i in 1..9999999 {
        let station = Station::new(format!("simple_station_{}", i), Production::new(
            vec![Amount::new(
                Product::PowerCells,
                1,
                10000,
                10000,
            )],
            vec![Amount::new(
                Product::Ores,
                2,
                0,
                20000,
            )],
            1,
            0,
        ));

        stations.insert(station.name().to_string(), station);
    }

    MyLittleUniverse::new(universe_name, TimeStackState::new(), stations, HashMap::new())
}