use crate::ExternalCommands::Time;
use crate::my_little_universe::MyLittleUniverse;
use crate::station::StationState;
use crate::time::TimeStackState;

pub fn generate_simple_universe(universe_name: String) -> MyLittleUniverse {
    MyLittleUniverse::new(universe_name, TimeStackState::new(), StationState::test_station())
}