use std::{thread, time};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use crate::station::{StationEvenType, StationState};
use crate::time::{TimeEventReturnType, TimeEventType, TimeStackState};

pub struct MyLittleUniverse {
    time: TimeStackState,
    channels: Vec<Channel>,
    station: StationState,
}

impl MyLittleUniverse {
    pub fn new() -> Self {
        MyLittleUniverse {
            time: TimeStackState::new(),
            channels: Vec::new(),
            station: StationState::test_station(),
        }
    }
}

// channel_getter is one channel to receive new channels.
// Then the loop will listen for events from that channel to execute.
fn game_loop(channel_getter: Receiver<Channel>) {
    thread::spawn(move || {
        let mut universe = MyLittleUniverse::new();

        loop {
            for channel in channel_getter.try_recv() {
                universe.channels.push(channel);
            }

            for channel in &universe.channels {
                for event in channel.getter.try_recv() {
                    match channel.returner.send(universe.time.push_event(&event)) {
                        Err(e) => eprintln!("Failed sending event in gameloop: {}", e),
                        _ => {}
                    }
                }
            }

            if universe.time.request_execute_turn() {
                universe.station.push_event(&StationEvenType::ExecuteTurn);
            }

            thread::sleep(time::Duration::from_millis(10))
        }
    });
}

pub struct Communicator {
    channel_sender: Sender<Channel>,
}

pub struct Channel {
    getter: Receiver<TimeEventType>,
    returner: Sender<TimeEventReturnType>,
}

impl Communicator {
    pub fn new() -> Self {
        let (channel_sender, channel_getter): (Sender<Channel>, Receiver<Channel>) = mpsc::channel();
        let stack = Communicator { channel_sender };
        game_loop(channel_getter);
        stack
    }
}

#[cfg(test)]
mod tests_int {
    use std::sync::mpsc;
    use std::sync::mpsc::{Receiver, Sender};
    use std::thread;
    use std::time::Duration;

    use crate::gameloop::{Channel, Communicator};
    use crate::time::{TimeEventReturnType, TimeEventType};
    use crate::time::TimeEventReturnType::{Received, StackState};

    #[test]
    fn it_works() {
        let (main_to_universe_sender, main_to_universe_receiver): (Sender<TimeEventType>, Receiver<TimeEventType>) = mpsc::channel();
        let (universe_to_main_sender, universe_to_main_receiver): (Sender<TimeEventReturnType>, Receiver<TimeEventReturnType>) = mpsc::channel();

        let time_stack = Communicator::new();
        let channel = Channel {
            getter: main_to_universe_receiver,
            returner: universe_to_main_sender,
        };
        match time_stack.channel_sender.send(channel) {
            Err(e) => println!("Sender errored: {}", e),
            _ => println!("Sender send without error.")
        }

        match main_to_universe_sender.send(TimeEventType::ReadyForNextTurn) {
            Err(e) => println!("Sender errored: {}", e),
            _ => println!("Sender send without error.")
        }

        await_recived(&universe_to_main_receiver);
    }

    #[test]
    fn next_turn() {
        let (main_to_universe_sender, main_to_universe_receiver): (Sender<TimeEventType>, Receiver<TimeEventType>) = mpsc::channel();
        let (universe_to_main_sender, universe_to_main_receiver): (Sender<TimeEventReturnType>, Receiver<TimeEventReturnType>) = mpsc::channel();

        let time_stack = Communicator::new();
        let channel = Channel {
            getter: main_to_universe_receiver,
            returner: universe_to_main_sender,
        };
        match time_stack.channel_sender.send(channel) {
            Err(e) => println!("Sender errored: {}", e),
            _ => println!("Sender send without error.")
        }

        check_turn(&main_to_universe_sender, &universe_to_main_receiver, 0);
        send_and_wait(&main_to_universe_sender, &universe_to_main_receiver, TimeEventType::ReadyForNextTurn);
        send_and_wait(&main_to_universe_sender, &universe_to_main_receiver, TimeEventType::Start);
        check_turn(&main_to_universe_sender, &universe_to_main_receiver, 1);
        send_and_wait(&main_to_universe_sender, &universe_to_main_receiver, TimeEventType::SetSpeed(1000));
        send_and_wait(&main_to_universe_sender, &universe_to_main_receiver, TimeEventType::ReadyForNextTurn);
        check_turn(&main_to_universe_sender, &universe_to_main_receiver, 1);
        thread::sleep(Duration::from_secs(1));
        check_turn(&main_to_universe_sender, &universe_to_main_receiver, 2);
        send_and_wait(&main_to_universe_sender, &universe_to_main_receiver, TimeEventType::SetSpeed(0));
        send_and_wait(&main_to_universe_sender, &universe_to_main_receiver, TimeEventType::Pause);
        send_and_wait(&main_to_universe_sender, &universe_to_main_receiver, TimeEventType::ReadyForNextTurn);
        check_turn(&main_to_universe_sender, &universe_to_main_receiver, 2);
        send_and_wait(&main_to_universe_sender, &universe_to_main_receiver, TimeEventType::Start);
        check_turn(&main_to_universe_sender, &universe_to_main_receiver, 3);
    }

    fn send_and_wait(main_to_universe_sender: &Sender<TimeEventType>, universe_to_main_receiver: &Receiver<TimeEventReturnType>, event_type: TimeEventType) {
        match main_to_universe_sender.send(event_type) {
            Err(e) => println!("Sender errored: {}", e),
            _ => println!("Sender send without error.")
        }

        await_recived(&universe_to_main_receiver);
    }

    fn check_turn(main_to_universe_sender: &Sender<TimeEventType>, universe_to_main_receiver: &Receiver<TimeEventReturnType>, expected_turn_count: u128) {
        match main_to_universe_sender.send(TimeEventType::GetTimeStackState) {
            Err(e) => println!("Sender errored: {}", e),
            _ => println!("Sender send without error.")
        }

        match universe_to_main_receiver.recv_timeout(Duration::from_secs(1)).unwrap() {
            StackState(state) => assert_eq!(expected_turn_count, state.turn()),
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