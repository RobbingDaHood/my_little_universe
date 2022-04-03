use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum TimeEventType {
    Internal(InternalTimeEventType),
    External(ExternalTimeEventType),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum InternalTimeEventType {
    ReadyForNextTurn,
    StartedNextTurn,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ExternalTimeEventType {
    Pause,
    Start,
    StartUntilTurn(u64),
    SetSpeed(u64),
    GetTimeStackState { include_stack: bool },
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct TimeStackState {
    turn: u64,
    turn_min_duration_in_milli_secs: u64,
    last_turn_timestamp: u64,
    last_processed_event_index: usize,
    pause_at_turn: Option<u64>,
    paused: bool,
    ready_for_next_turn: bool,
    event_stack: Vec<TimeEventType>,
}

impl TimeStackState {
    pub fn new() -> Self {
        TimeStackState {
            turn: 0,
            turn_min_duration_in_milli_secs: 0,
            last_turn_timestamp: Self::epcoh_time(),
            last_processed_event_index: 0,
            paused: true,
            ready_for_next_turn: true,
            event_stack: Vec::new(),
            pause_at_turn: Option::None,
        }
    }

    fn epcoh_time() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis()
            .try_into()
            .unwrap()
    }

    pub fn push_event(&mut self, event: &TimeEventType) -> TimeEventReturnType {
        // self.event_stack.push(event.clone());
        self.handle_event(event)
    }

    fn handle_event(&mut self, event: &TimeEventType) -> TimeEventReturnType {
        match event {
            TimeEventType::Internal(internal_event) => {
                match internal_event {
                    InternalTimeEventType::ReadyForNextTurn => {
                        self.ready_for_next_turn = true;
                        TimeEventReturnType::Received
                    }
                    InternalTimeEventType::StartedNextTurn => {
                        self.next_turn();
                        TimeEventReturnType::Received
                    }
                }
            }

            TimeEventType::External(external_event) => {
                match external_event {
                    ExternalTimeEventType::Pause => {
                        self.paused = true;
                        TimeEventReturnType::Received
                    }
                    ExternalTimeEventType::Start => {
                        self.paused = false;
                        TimeEventReturnType::Received
                    }
                    ExternalTimeEventType::SetSpeed(turn_min_duration_in_milli_secs) => {
                        self.turn_min_duration_in_milli_secs = *turn_min_duration_in_milli_secs;
                        TimeEventReturnType::Received
                    }
                    ExternalTimeEventType::GetTimeStackState { include_stack } => {
                        if *include_stack {
                            TimeEventReturnType::StackState(self.clone())
                        } else {
                            let mut state = self.clone();
                            state.event_stack = Vec::new();
                            TimeEventReturnType::StackState(state)
                        }
                    }
                    ExternalTimeEventType::StartUntilTurn(turn) => {
                        self.pause_at_turn = Option::Some(*turn);
                        self.paused = false;
                        TimeEventReturnType::Received
                    }
                }
            }
        }
    }

    pub fn request_execute_turn(&mut self) -> bool {
        if self.ready_for_next_turn() && !self.paused() {
            let now = Self::epcoh_time();
            let min_instant_where_we_can_switch_turn = self.last_turn_timestamp().checked_add(self.turn_min_duration_in_milli_secs() as u64).unwrap();

            if now >= min_instant_where_we_can_switch_turn {
                self.push_event(&TimeEventType::Internal(InternalTimeEventType::StartedNextTurn));
                return true;
            }
        }
        false
    }

    fn next_turn(&mut self) {
        self.turn += 1;
        self.ready_for_next_turn = false;
        self.last_turn_timestamp = Self::epcoh_time();

        if let Some(pause_at_turn) = self.pause_at_turn {
            if self.turn >= pause_at_turn {
                self.paused = true;
                self.pause_at_turn = Option::None;
            }
        }
    }
    pub fn turn(&self) -> u64 {
        self.turn
    }
    pub fn turn_min_duration_in_milli_secs(&self) -> u64 {
        self.turn_min_duration_in_milli_secs
    }
    pub fn last_turn_timestamp(&self) -> u64 {
        self.last_turn_timestamp
    }
    pub fn paused(&self) -> bool {
        self.paused
    }
    pub fn ready_for_next_turn(&self) -> bool {
        self.ready_for_next_turn
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum TimeEventReturnType {
    StackState(TimeStackState),
    Received,
}

#[cfg(test)]
mod tests_int {
    use serde_json::json;

    use crate::time::*;

    #[test]
    fn handle_events() {
        let mut time_state = TimeStackState::new();

        match time_state.push_event(&TimeEventType::External(ExternalTimeEventType::GetTimeStackState { include_stack: true })) {
            TimeEventReturnType::StackState(state) => {
                assert_eq!(0, state.turn);
                assert_eq!(true, state.ready_for_next_turn);
            }
            _ => assert!(false)
        }

        match time_state.push_event(&TimeEventType::Internal(InternalTimeEventType::ReadyForNextTurn)) {
            TimeEventReturnType::Received => {}
            _ => assert!(false)
        }

        match time_state.push_event(&TimeEventType::External(ExternalTimeEventType::GetTimeStackState { include_stack: true })) {
            TimeEventReturnType::StackState(state) => {
                assert_eq!(0, state.turn);
                assert_eq!(true, state.ready_for_next_turn);
            }
            _ => assert!(false)
        }
    }

    #[test]
    fn change_turn() {
        let mut time_state = TimeStackState::new();
        assert_eq!(0, time_state.turn);
        time_state.next_turn();
        assert_eq!(1, time_state.turn);
    }

    #[test]
    fn serialise_deserialize() {
        let mut time_state = TimeStackState::new();
        push_all_events(&mut time_state);

        let json = json!(time_state).to_string();
        let and_back = serde_json::from_str(&json).unwrap();
        assert_eq!(time_state, and_back);
    }

    fn push_all_events(time_state: &mut TimeStackState) {
        match time_state.push_event(&TimeEventType::Internal(InternalTimeEventType::ReadyForNextTurn)) {
            TimeEventReturnType::Received => {}
            _ => assert!(false)
        }
        match time_state.push_event(&TimeEventType::Internal(InternalTimeEventType::StartedNextTurn)) {
            TimeEventReturnType::Received => {}
            _ => assert!(false)
        }
        match time_state.push_event(&TimeEventType::External(ExternalTimeEventType::Start)) {
            TimeEventReturnType::Received => {}
            _ => assert!(false)
        }
        match time_state.push_event(&TimeEventType::External(ExternalTimeEventType::Pause)) {
            TimeEventReturnType::Received => {}
            _ => assert!(false)
        }
        match time_state.push_event(&TimeEventType::External(ExternalTimeEventType::GetTimeStackState { include_stack: true })) {
            TimeEventReturnType::StackState(_) => {}
            _ => assert!(false)
        }
        match time_state.push_event(&TimeEventType::External(ExternalTimeEventType::SetSpeed(1000))) {
            TimeEventReturnType::Received => {}
            _ => assert!(false)
        }
        match time_state.push_event(&TimeEventType::External(ExternalTimeEventType::StartUntilTurn(1000))) {
            TimeEventReturnType::Received => {}
            _ => assert!(false)
        }
    }
}