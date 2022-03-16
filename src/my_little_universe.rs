use std::collections::HashMap;
use crate::{Channel, ExternalCommandReturnValues, ExternalCommands};
use crate::save_load::ExternalSaveLoad;
use crate::station::{InternalStationEventType, StationEvenReturnType, StationEventType, Station};
use crate::time::{InternalTimeEventType, TimeEventReturnType, TimeEventType, TimeStackState};
use serde::{Deserialize, Serialize};

pub struct MyLittleUniverse {
    time: TimeStackState,
    constructs: HashMap<String, Station>,
    universe_name: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum MyLittleUniverseReturnValues {
    CouldNotFindStation
}

impl MyLittleUniverse {
    pub fn new(universe_name: String, time: TimeStackState, constructs: HashMap<String, Station>) -> Self {
        MyLittleUniverse {
            time: time,
            constructs,
            universe_name,
        }
    }

    pub fn time(&self) -> &TimeStackState {
        &self.time
    }

    pub fn constructs(&self) -> &HashMap<String, Station> {
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
                return match self.constructs.get_mut(&station_name) {
                    Some(construct) => {
                        let return_type = construct.push_event(&StationEventType::External(station_event));
                        ExternalCommandReturnValues::Station(return_type)
                    }
                    None => { ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::CouldNotFindStation) }
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
            for construct in self.constructs.values_mut() {
                construct.push_event(&StationEventType::Internal(InternalStationEventType::ExecuteTurn));
            }
            self.time.push_event(&TimeEventType::Internal(InternalTimeEventType::ReadyForNextTurn));
        }
    }
}