use crate::time::{ExternalTimeEventType, TimeEventReturnType};

pub enum ExternalCommands {
    Time(ExternalTimeEventType)
}

pub enum ExternalCommandReturnValues {
    Time(TimeEventReturnType)
}
