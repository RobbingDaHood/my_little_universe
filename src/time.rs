use std::{thread, time};
use std::ops::Add;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

pub struct MyLittleUniverse {
    channel_sender: Sender<Channel>,
}

pub struct Channel {
    getter: Receiver<TimeEventType>,
    returner: Sender<TimeEventReturnType>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum TimeEventType {
    ReadyForNextTurn,
    Pause,
    Start,
    SetSpeed(u64),
}

#[derive(Clone, PartialEq, Debug)]
pub enum TimeEventReturnType {
    StackState(TimeStackState)
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

fn time_passes(channel_getter: Receiver<Channel>) {
    thread::spawn(move || {
        println!("Initiated gammeloop");
        let mut event_stack: Vec<TimeEventType> = Vec::new();
        let mut channels: Vec<Channel> = Vec::new();
        let mut state = TimeStackState {
            turn: 0,
            turn_min_duration_in_milli_secs: 1000,
            last_turn_timestamp: Instant::now(),
            last_processed_event_index: 0,
            paused: true,
            ready_for_next_turn: false,
        };

        loop {
            for channel in channel_getter.try_recv() {
                channels.push(channel);
            }

            println!("gameloop: next itteration. NUmber of channelse: {}", channels.len());
            for channel in &channels {
                for event in channel.getter.try_recv() {
                    println!("Got event, handling: {:?}", event);
                    handle_event(&mut state, &event, channel);
                    event_stack.push(event);
                }
            }

            if state.ready_for_next_turn && !state.paused {
                let now = Instant::now();
                let min_instant_where_we_can_switch_turn = state.last_turn_timestamp.add(Duration::from_millis(state.turn_min_duration_in_milli_secs));

                if now > min_instant_where_we_can_switch_turn {
                    state.turn += 1;
                    // Trigger next turn calculation
                    println!("Started turn {}", state.turn);
                    state.ready_for_next_turn = false;
                }
            }

            thread::sleep(time::Duration::from_millis(10))
        }
    });
}

fn handle_event(state: &mut TimeStackState, event: &TimeEventType, channel: &Channel) {
    match event {
        TimeEventType::Pause => {
            state.paused = true;
            channel.returner.send(TimeEventReturnType::StackState(state.clone()));
        }
        TimeEventType::Start => {
            state.paused = false;
        }
        TimeEventType::SetSpeed(turn_min_duration_in_milli_secs) => {
            state.turn_min_duration_in_milli_secs = *turn_min_duration_in_milli_secs;
        }
        TimeEventType::ReadyForNextTurn => {
            println!("ReadyForNextTurn triggered.");
            state.ready_for_next_turn = true;
            channel.returner.send(TimeEventReturnType::StackState(state.clone()));
        }
    }
}

impl MyLittleUniverse {
    pub fn new() -> Self {
        let (channel_sender, channel_getter): (Sender<Channel>, Receiver<Channel>) = mpsc::channel();
        let stack = MyLittleUniverse { channel_sender };
        time_passes(channel_getter);
        stack
    }
}


#[cfg(test)]
mod tests_int {
    use std::sync::mpsc;
    use std::sync::mpsc::{Receiver, Sender, SendError};
    use std::time::{Duration, Instant};

    use crate::time::{Channel, MyLittleUniverse, TimeEventReturnType, TimeEventType, TimeStackState};
    use crate::time::TimeEventReturnType::StackState;

    #[test]
    fn it_works() {
        let (main_to_universe_sender, main_to_universe_reciver): (Sender<TimeEventType>, Receiver<TimeEventType>) = mpsc::channel();
        let (universe_to_main_sender, universe_to_main_reciver): (Sender<TimeEventReturnType>, Receiver<TimeEventReturnType>) = mpsc::channel();

        let time_stack = MyLittleUniverse::new();
        time_stack.channel_sender.send(Channel {
            getter: main_to_universe_reciver,
            returner: universe_to_main_sender,
        });

        match main_to_universe_sender.send(TimeEventType::ReadyForNextTurn) {
            Err(E) => println!("Sender errored: {}", E),
            _  => println!("Sender send without error.")
        }

        match universe_to_main_reciver.recv().unwrap() {
            StackState(state) => assert_eq!(true, state.ready_for_next_turn)
        }
    }
}
