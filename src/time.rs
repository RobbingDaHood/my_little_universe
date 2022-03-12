use std::ops::Add;
use std::time::{Duration, Instant};

#[derive(Clone, PartialEq, Debug)]
pub enum TimeEventType {
    Internal(InternalTimeEventType),
    External(ExternalTimeEventType)
}

#[derive(Clone, PartialEq, Debug)]
pub enum InternalTimeEventType {
    ReadyForNextTurn,
    StartedNextTurn,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ExternalTimeEventType {
    Pause,
    Start,
    StartUntilTurn(u128),
    SetSpeed(u64),
    GetTimeStackState,
}

#[derive(Clone, PartialEq, Debug)]
pub struct TimeStackState {
    turn: u128,
    turn_min_duration_in_milli_secs: u64,
    last_turn_timestamp: Instant,
    last_processed_event_index: usize,
    pause_at_turn: Option<u128>,
    paused: bool,
    ready_for_next_turn: bool,
    event_stack: Vec<TimeEventType>,
}

impl TimeStackState {
    pub fn new() -> Self {
        TimeStackState {
            turn: 0,
            turn_min_duration_in_milli_secs: 0,
            last_turn_timestamp: Instant::now(),
            last_processed_event_index: 0,
            paused: true,
            ready_for_next_turn: true,
            event_stack: Vec::new(),
            pause_at_turn: Option::None,
        }
    }

    pub fn turn_min_duration_in_milli_secs(&self) -> u64 {
        self.turn_min_duration_in_milli_secs
    }
    pub fn last_turn_timestamp(&self) -> Instant {
        self.last_turn_timestamp
    }
    pub fn paused(&self) -> bool {
        self.paused
    }
    pub fn ready_for_next_turn(&self) -> bool {
        self.ready_for_next_turn
    }

    pub fn push_event(&mut self, event: &TimeEventType) -> TimeEventReturnType {
        self.event_stack.push(event.clone());
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
                    ExternalTimeEventType::GetTimeStackState => {
                        TimeEventReturnType::StackState(self.clone())
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
            let now = Instant::now();
            let min_instant_where_we_can_switch_turn = self.last_turn_timestamp().add(Duration::from_millis(self.turn_min_duration_in_milli_secs()));

            if now > min_instant_where_we_can_switch_turn {
                self.push_event(&TimeEventType::Internal(InternalTimeEventType::StartedNextTurn));
                return true;
            }
        }
        false
    }

    fn next_turn(&mut self) {
        self.turn += 1;
        self.ready_for_next_turn = false;
        self.last_turn_timestamp = Instant::now();

        if let Some(pause_at_turn) = self.pause_at_turn {
            if self.turn >= pause_at_turn {
                self.paused = true;
                self.pause_at_turn = Option::None;
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum TimeEventReturnType {
    StackState(TimeStackState),
    Received,
}

#[cfg(test)]
mod tests_int {
    use crate::time::*;

    #[test]
    fn handle_events() {
        let mut time_state = TimeStackState::new();

        match time_state.push_event(&TimeEventType::External(ExternalTimeEventType::GetTimeStackState)) {
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

        match time_state.push_event(&TimeEventType::External(ExternalTimeEventType::GetTimeStackState)) {
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
}