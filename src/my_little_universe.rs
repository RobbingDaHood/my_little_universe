use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{Channel, ExternalCommandReturnValues, ExternalCommands};
use crate::construct::construct::{Construct, ConstructEventType, InternalConstructEventType};
use crate::construct_module::ConstructModuleType;
use crate::save_load::ExternalSaveLoad;
use crate::station::{InternalStationEventType, Station, StationEvenReturnType, StationEventType};
use crate::time::{InternalTimeEventType, TimeEventReturnType, TimeEventType, TimeStackState};

pub struct MyLittleUniverse {
    time: TimeStackState,
    stations: HashMap<String, Station>,
    constructs: HashMap<String, Construct>,
    universe_name: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum MyLittleUniverseReturnValues {
    CouldNotFindStation,
    CouldNotFindConstruct,
}

impl MyLittleUniverse {
    pub fn new(universe_name: String, time: TimeStackState, stations: HashMap<String, Station>, constructs: HashMap<String, Construct>) -> Self {
        MyLittleUniverse {
            time,
            stations,
            universe_name,
            constructs,
        }
    }

    pub fn time(&self) -> &TimeStackState {
        &self.time
    }

    pub fn stations(&self) -> &HashMap<String, Station> {
        &self.stations
    }
    pub fn constructs(&self) -> &HashMap<String, Construct> {
        &self.constructs
    }
    pub fn universe_name(&self) -> &str {
        &self.universe_name
    }

    pub fn handle_event(&mut self, event: ExternalCommands) -> ExternalCommandReturnValues {
        match event {
            ExternalCommands::Time(time_event) => {
                let return_type = self.time.push_event(&TimeEventType::External(time_event));
                ExternalCommandReturnValues::Time(return_type)
            }
            ExternalCommands::Station(station_name, station_event) => {
                return match self.stations.get_mut(&station_name) {
                    Some(station) => {
                        let return_type = station.push_event(&StationEventType::External(station_event));
                        ExternalCommandReturnValues::Station(return_type)
                    }
                    None => { ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::CouldNotFindStation) }
                };
            }
            ExternalCommands::Construct(construct_name, construct_event) => {
                return match self.constructs.get_mut(&construct_name) {
                    Some(construct) => {
                        let return_type = construct.push_event(&ConstructEventType::External(construct_event));
                        ExternalCommandReturnValues::Construct(return_type)
                    }
                    None => { ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::CouldNotFindConstruct) }
                };
            }
            ExternalCommands::Save(save_event) => {
                match save_event {
                    ExternalSaveLoad::SaveTheUniverseAs(universe_name) => {
                        ExternalCommandReturnValues::Save(self.save_as(&universe_name))
                    }
                    ExternalSaveLoad::SaveTheUniverse => {
                        ExternalCommandReturnValues::Save(self.save())
                    }
                }
            }
        }
    }

    pub fn request_execute_turn(&mut self) {
        if self.time.request_execute_turn() {
            for station in self.stations.values_mut() {
                station.push_event(&StationEventType::Internal(InternalStationEventType::ExecuteTurn(self.time.turn())));
            }
            for construct in self.constructs.values_mut() {
                construct.push_event(&ConstructEventType::Internal(InternalConstructEventType::ExecuteTurn(self.time.turn())));
            }
            self.time.push_event(&TimeEventType::Internal(InternalTimeEventType::ReadyForNextTurn));
        }
    }
}