use crate::station::{ExternalStationEventType, StationEvenReturnType};
use crate::time::{ExternalTimeEventType, TimeEventReturnType};

pub enum ExternalCommands {
    Time(ExternalTimeEventType),
    Station(ExternalStationEventType),
}

pub enum ExternalCommandReturnValues {
    Time(TimeEventReturnType),
    Station(StationEvenReturnType),
}
