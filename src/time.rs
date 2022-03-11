use std::time::Instant;

#[derive(Clone, PartialEq, Debug)]
pub enum TimeEventType {
    ReadyForNextTurn,
    Pause,
    Start,
    SetSpeed(u64),
    GetTimeStackState,
}

#[derive(Clone, PartialEq, Debug)]
pub struct TimeStackState {
    turn: u128,
    turn_min_duration_in_milli_secs: u64,
    last_turn_timestamp: Instant,
    last_processed_event_index: usize,
    paused: bool,
    ready_for_next_turn: bool,
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
}

#[derive(Clone, PartialEq, Debug)]
pub enum TimeEventReturnType {
    StackState(TimeStackState),
    Received,
}

pub fn handle_event(state: &mut TimeStackState, event: &TimeEventType) -> TimeEventReturnType {
    match event {
        TimeEventType::Pause => {
            state.paused = true;
            TimeEventReturnType::Received
        }
        TimeEventType::Start => {
            state.paused = false;
            TimeEventReturnType::Received
        }
        TimeEventType::SetSpeed(turn_min_duration_in_milli_secs) => {
            state.turn_min_duration_in_milli_secs = *turn_min_duration_in_milli_secs;
            TimeEventReturnType::Received
        }
        TimeEventType::ReadyForNextTurn => {
            state.ready_for_next_turn = true;
            TimeEventReturnType::Received
        }
        TimeEventType::GetTimeStackState => {
            TimeEventReturnType::StackState(state.clone())
        }
    }
}

pub fn next_turn(state: &mut TimeStackState) {
    state.turn += 1;
    state.ready_for_next_turn = false;
}


#[cfg(test)]
mod tests_int {
    use crate::time::*;

    #[test]
    fn handl_events() {
        let mut time_state = TimeStackState::new();

        match handle_event(&mut time_state, &TimeEventType::GetTimeStackState) {
            TimeEventReturnType::StackState(state) => {
                assert_eq!(0, state.turn);
                assert_eq!(false, state.ready_for_next_turn);
            }
            _ => assert!(false)
        }

        match handle_event(&mut time_state, &TimeEventType::ReadyForNextTurn) {
            TimeEventReturnType::Received => {}
            _ => assert!(false)
        }

        match handle_event(&mut time_state, &TimeEventType::GetTimeStackState) {
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
        next_turn(&mut time_state);
        assert_eq!(1, time_state.turn);

    }
}