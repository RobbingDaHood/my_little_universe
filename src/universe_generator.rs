use std::collections::HashMap;
use crate::my_little_universe::MyLittleUniverse;
use crate::products::Product;
use crate::station::StationEvenReturnType::StationState;
use crate::station::{Amount, Production, Station};
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

    let mut stations : HashMap<String, Station> = HashMap::new();
    stations.insert(station.name().to_string(), station);

    MyLittleUniverse::new(universe_name, TimeStackState::new(), stations)
}