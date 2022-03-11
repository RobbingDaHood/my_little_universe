use std::ops::Add;
use std::time::{Duration, Instant};

#[derive(Clone, PartialEq, Debug)]
pub enum TimeEventType {
    ReadyForNextTurn,
    Pause,
    Start,
    SetSpeed(u64),
    GetTimeStackState,
    StartedNextTurn,
}

#[derive(Clone, PartialEq, Debug)]
pub struct TimeStackState {
    turn: u128,
    turn_min_duration_in_milli_secs: u64,
    last_turn_timestamp: Instant,
    last_processed_event_index: usize,
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
            ready_for_next_turn: false,
            event_stack: Vec::new(),
        }
    }

    pub fn turn(&self) -> u128 {
        self.turn
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
            TimeEventType::Pause => {
                self.paused = true;
                TimeEventReturnType::Received
            }
            TimeEventType::Start => {
                self.paused = false;
                TimeEventReturnType::Received
            }
            TimeEventType::SetSpeed(turn_min_duration_in_milli_secs) => {
                self.turn_min_duration_in_milli_secs = *turn_min_duration_in_milli_secs;
                TimeEventReturnType::Received
            }
            TimeEventType::ReadyForNextTurn => {
                self.ready_for_next_turn = true;
                TimeEventReturnType::Received
            }
            TimeEventType::GetTimeStackState => {
                TimeEventReturnType::StackState(self.clone())
            }
            TimeEventType::StartedNextTurn => {
                self.next_turn();
                TimeEventReturnType::Received
            }
        }
    }

    pub fn request_execute_turn(&mut self) -> bool {
        if self.ready_for_next_turn() && !self.paused() {
            let now = Instant::now();
            let min_instant_where_we_can_switch_turn = self.last_turn_timestamp().add(Duration::from_millis(self.turn_min_duration_in_milli_secs()));

            if now > min_instant_where_we_can_switch_turn {
                self.push_event(&TimeEventType::StartedNextTurn);
                return true;
            }
        }
        false
    }

    fn next_turn(&mut self) {
        self.turn += 1;
        self.ready_for_next_turn = false;
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
    fn handl_events() {
        let mut time_state = TimeStackState::new();

        match time_state.push_event(&TimeEventType::GetTimeStackState) {
            TimeEventReturnType::StackState(state) => {
                assert_eq!(0, state.turn);
                assert_eq!(false, state.ready_for_next_turn);
            }
            _ => assert!(false)
        }

        match time_state.push_event(&TimeEventType::ReadyForNextTurn) {
            TimeEventReturnType::Received => {}
            _ => assert!(false)
        }

        match time_state.push_event(&TimeEventType::GetTimeStackState) {
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