use crate::{Channel, ExternalCommandReturnValues, ExternalCommands};
use crate::save_load::ExternalSaveLoad;
use crate::station::{InternalStationEventType, StationEvenReturnType, StationEventType, StationState};
use crate::time::{InternalTimeEventType, TimeEventReturnType, TimeEventType, TimeStackState};

pub struct MyLittleUniverse {
    pub time: TimeStackState,
    pub station: StationState,
    pub universe_name: String,
}

impl MyLittleUniverse {
    pub fn new(universe_name: String, time: TimeStackState, station: StationState) -> Self {
        MyLittleUniverse {
            time: time,
            station: station,
            universe_name,
        }
    }

    pub fn time(&self) -> &TimeStackState {
        &self.time
    }

    pub fn station(&self) -> &StationState {
        &self.station
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
            ExternalCommands::Station(station_event) => {
                let return_type = self.station.push_event(&StationEventType::External(station_event));
                ExternalCommandReturnValues::Station(return_type)
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
            self.station.push_event(&StationEventType::Internal(InternalStationEventType::ExecuteTurn));
            self.time.push_event(&TimeEventType::Internal(InternalTimeEventType::ReadyForNextTurn));
        }
    }
}