use std::{thread, time};
use std::ops::Add;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};
use crate::time::{handle_event, next_turn, TimeEventReturnType, TimeEventType, TimeStackState};

pub struct Channel {
    getter: Receiver<TimeEventType>,
    returner: Sender<TimeEventReturnType>,
}

fn game_loop(channel_getter: Receiver<Channel>) {
    thread::spawn(move || {
        let mut event_stack: Vec<TimeEventType> = Vec::new();
        let mut channels: Vec<Channel> = Vec::new();
        let mut state = TimeStackState::new();

        loop {
            for channel in channel_getter.try_recv() {
                channels.push(channel);
            }

            for channel in &channels {
                for event in channel.getter.try_recv() {
                    channel.returner.send(handle_event(&mut state, &event));
                    event_stack.push(event);
                }
            }

            if state.ready_for_next_turn() && !state.paused() {
                let now = Instant::now();
                let min_instant_where_we_can_switch_turn = state.last_turn_timestamp().add(Duration::from_millis(state.turn_min_duration_in_milli_secs()));

                if now > min_instant_where_we_can_switch_turn {
                    // Trigger next turn calculation
                    next_turn(&mut state);
                }
            }

            thread::sleep(time::Duration::from_millis(10))
        }
    });
}

pub struct MyLittleUniverse {
    channel_sender: Sender<Channel>,
}

impl MyLittleUniverse {
    pub fn new() -> Self {
        let (channel_sender, channel_getter): (Sender<Channel>, Receiver<Channel>) = mpsc::channel();
        let stack = MyLittleUniverse { channel_sender };
        game_loop(channel_getter);
        stack
    }
}

#[cfg(test)]
mod tests_int {
    use std::sync::mpsc;
    use std::sync::mpsc::{Receiver, Sender, SendError};
    use std::thread;
    use std::time::{Duration, Instant};
    use crate::gameloop::{Channel, MyLittleUniverse};

    use crate::time;
    use crate::time::TimeEventReturnType::{Received, StackState};
    use crate::time::{TimeEventReturnType, TimeEventType};

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
            _ => println!("Sender send without error.")
        }

        await_recived(&universe_to_main_reciver);
    }

    #[test]
    fn next_turn() {
        let (main_to_universe_sender, main_to_universe_reciver): (Sender<TimeEventType>, Receiver<TimeEventType>) = mpsc::channel();
        let (universe_to_main_sender, universe_to_main_reciver): (Sender<TimeEventReturnType>, Receiver<TimeEventReturnType>) = mpsc::channel();

        let time_stack = MyLittleUniverse::new();
        time_stack.channel_sender.send(Channel {
            getter: main_to_universe_reciver,
            returner: universe_to_main_sender,
        });

        match main_to_universe_sender.send(TimeEventType::ReadyForNextTurn) {
            Err(E) => println!("Sender errored: {}", E),
            _ => println!("Sender send without error.")
        }

        await_recived(&universe_to_main_reciver);

        match main_to_universe_sender.send(TimeEventType::Start) {
            Err(E) => println!("Sender errored: {}", E),
            _ => println!("Sender send without error.")
        }

        await_recived(&universe_to_main_reciver);

        match main_to_universe_sender.send(TimeEventType::GetTimeStackState) {
            Err(E) => println!("Sender errored: {}", E),
            _ => println!("Sender send without error.")
        }

        match universe_to_main_reciver.recv_timeout(Duration::from_secs(1)).unwrap() {
            StackState(state) => assert_eq!(1, state.turn()),
            _ => assert!(false)
        }

        match main_to_universe_sender.send(TimeEventType::SetSpeed(1000)) {
            Err(E) => println!("Sender errored: {}", E),
            _ => println!("Sender send without error.")
        }

        await_recived(&universe_to_main_reciver);

        match main_to_universe_sender.send(TimeEventType::ReadyForNextTurn) {
            Err(E) => println!("Sender errored: {}", E),
            _ => println!("Sender send without error.")
        }

        await_recived(&universe_to_main_reciver);

        thread::sleep(Duration::from_secs(1));

        match main_to_universe_sender.send(TimeEventType::GetTimeStackState) {
            Err(E) => println!("Sender errored: {}", E),
            _ => println!("Sender send without error.")
        }

        match universe_to_main_reciver.recv_timeout(Duration::from_secs(1)).unwrap() {
            StackState(state) => assert_eq!(2, state.turn()),
            _ => assert!(false)
        }
    }

    fn await_recived(universe_to_main_reciver: &Receiver<TimeEventReturnType>) {
        match universe_to_main_reciver.recv_timeout(Duration::from_secs(1)).unwrap() {
            Received => { println!("Recived without error.") }
            _ => assert!(false)
        }
    }
}