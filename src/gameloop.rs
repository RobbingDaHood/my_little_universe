use std::{thread, time};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use crate::external_commands::{ExternalCommandReturnValues, ExternalCommands};
use crate::station::{InternalStationEventType, StationEventType, StationState};
use crate::time::{InternalTimeEventType, TimeEventType, TimeStackState};

pub struct MyLittleUniverse {
    time: TimeStackState,
    channels: Vec<Channel>,
    station: StationState,
    universe_name: String,
}

impl MyLittleUniverse {
    pub fn new(universe_name: String, time: TimeStackState, station: StationState) -> Self {
        MyLittleUniverse {
            time: time,
            channels: Vec::new(),
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
}

// channel_getter is one channel to receive new channels.
// Then the loop will listen for events from that channel to execute.
fn game_loop(channel_getter: Receiver<Channel>, universe_name: String) {
    thread::spawn(move || {
        let mut universe = MyLittleUniverse::new(universe_name.clone(), TimeStackState::new(), StationState::test_station());

        loop {
            for channel in channel_getter.try_recv() {
                universe.channels.push(channel);
            }

            for channel in &universe.channels {
                for event in channel.getter.try_recv() {
                    match event {
                        ExternalCommands::Time(time_event) => {
                            match channel.returner.send(ExternalCommandReturnValues::Time(universe.time.push_event(&TimeEventType::External(time_event)))) {
                                Err(e) => eprintln!("Failed sending event in gameloop: {}", e),
                                _ => {}
                            }
                        }
                        ExternalCommands::Station(station_event) => {
                            match channel.returner.send(ExternalCommandReturnValues::Station(universe.station.push_event(&StationEventType::External(station_event)))) {
                                Err(e) => eprintln!("Failed sending event in gameloop: {}", e),
                                _ => {}
                            }
                        }
                    }
                }
            }

            if universe.time.request_execute_turn() {
                universe.station.push_event(&StationEventType::Internal(InternalStationEventType::ExecuteTurn));
                universe.time.push_event(&TimeEventType::Internal(InternalTimeEventType::ReadyForNextTurn));
            }

            thread::sleep(time::Duration::from_millis(10))
        }
    });
}

pub struct Communicator {
    channel_sender: Sender<Channel>,
}

impl Communicator {
    pub fn channel_sender(&self) -> &Sender<Channel> {
        &self.channel_sender
    }
}

pub struct Channel {
    getter: Receiver<ExternalCommands>,
    returner: Sender<ExternalCommandReturnValues>,
}

impl Channel {
    pub fn new(getter: Receiver<ExternalCommands>, returner: Sender<ExternalCommandReturnValues>) -> Self {
        Channel { getter, returner }
    }
}

impl Communicator {
    pub fn new(universe_name: String) -> Self {
        let (channel_sender, channel_getter): (Sender<Channel>, Receiver<Channel>) = mpsc::channel();
        let stack = Communicator { channel_sender };
        game_loop(channel_getter, universe_name);
        stack
    }
}

#[cfg(test)]
mod tests_int {
    use std::sync::mpsc;
    use std::sync::mpsc::{Receiver, Sender};
    use std::thread;
    use std::time::Duration;

    use crate::external_commands::{ExternalCommandReturnValues, ExternalCommands};
    use crate::gameloop::{Channel, Communicator};
    use crate::products::Product;
    use crate::station::{ExternalStationEventType, LoadingRequest};
    use crate::station::StationEvenReturnType::{Approved, StationState};
    use crate::time::ExternalTimeEventType;
    use crate::time::TimeEventReturnType::{Received, StackState};

    #[test]
    fn it_works() {
        let (main_to_universe_sender, main_to_universe_receiver): (Sender<ExternalCommands>, Receiver<ExternalCommands>) = mpsc::channel();
        let (universe_to_main_sender, universe_to_main_receiver): (Sender<ExternalCommandReturnValues>, Receiver<ExternalCommandReturnValues>) = mpsc::channel();

        let time_stack = Communicator::new("testing".to_string());
        let channel = Channel {
            getter: main_to_universe_receiver,
            returner: universe_to_main_sender,
        };
        match time_stack.channel_sender.send(channel) {
            Err(e) => println!("Sender errored: {}", e),
            _ => {}
        }

        match main_to_universe_sender.send(ExternalCommands::Time(ExternalTimeEventType::Pause)) {
            Err(e) => println!("Sender errored: {}", e),
            _ => {}
        }

        await_recived(&universe_to_main_receiver);
    }

    #[test]
    fn next_turn() {
        let (main_to_universe_sender, main_to_universe_receiver): (Sender<ExternalCommands>, Receiver<ExternalCommands>) = mpsc::channel();
        let (universe_to_main_sender, universe_to_main_receiver): (Sender<ExternalCommandReturnValues>, Receiver<ExternalCommandReturnValues>) = mpsc::channel();

        let time_stack = Communicator::new("testing".to_string());
        let channel = Channel {
            getter: main_to_universe_receiver,
            returner: universe_to_main_sender,
        };
        match time_stack.channel_sender.send(channel) {
            Err(e) => println!("Sender errored: {}", e),
            _ => {}
        }

        check_turn(&main_to_universe_sender, &universe_to_main_receiver, 0);
        send_and_wait(&main_to_universe_sender, &universe_to_main_receiver, ExternalCommands::Time(ExternalTimeEventType::StartUntilTurn(1)));
        check_turn(&main_to_universe_sender, &universe_to_main_receiver, 1);
        send_and_wait(&main_to_universe_sender, &universe_to_main_receiver, ExternalCommands::Time(ExternalTimeEventType::SetSpeed(1000)));
        send_and_wait(&main_to_universe_sender, &universe_to_main_receiver, ExternalCommands::Time(ExternalTimeEventType::StartUntilTurn(2)));
        check_turn(&main_to_universe_sender, &universe_to_main_receiver, 1);
        thread::sleep(Duration::from_secs(1));
        check_turn(&main_to_universe_sender, &universe_to_main_receiver, 2);
        send_and_wait(&main_to_universe_sender, &universe_to_main_receiver, ExternalCommands::Time(ExternalTimeEventType::SetSpeed(0)));
        send_and_wait(&main_to_universe_sender, &universe_to_main_receiver, ExternalCommands::Time(ExternalTimeEventType::StartUntilTurn(3)));
        check_turn(&main_to_universe_sender, &universe_to_main_receiver, 3);
    }

    #[test]
    fn next_turn_without_limit() {
        let (main_to_universe_sender, main_to_universe_receiver): (Sender<ExternalCommands>, Receiver<ExternalCommands>) = mpsc::channel();
        let (universe_to_main_sender, universe_to_main_receiver): (Sender<ExternalCommandReturnValues>, Receiver<ExternalCommandReturnValues>) = mpsc::channel();

        let time_stack = Communicator::new("testing".to_string());
        let channel = Channel {
            getter: main_to_universe_receiver,
            returner: universe_to_main_sender,
        };
        match time_stack.channel_sender.send(channel) {
            Err(e) => println!("Sender errored: {}", e),
            _ => {}
        }

        check_turn(&main_to_universe_sender, &universe_to_main_receiver, 0);
        send_and_wait(&main_to_universe_sender, &universe_to_main_receiver, ExternalCommands::Time(ExternalTimeEventType::Start));
        thread::sleep(Duration::from_secs(1));
        send_and_wait(&main_to_universe_sender, &universe_to_main_receiver, ExternalCommands::Time(ExternalTimeEventType::Pause));
        match main_to_universe_sender.send(ExternalCommands::Time(ExternalTimeEventType::GetTimeStackState { include_stack: true })) {
            Err(e) => println!("Sender errored: {}", e),
            _ => {}
        }

        match universe_to_main_receiver.recv_timeout(Duration::from_secs(1)).unwrap() {
            ExternalCommandReturnValues::Time(time_return) => {
                match time_return {
                    StackState(state) => assert!(10 < state.turn()),
                    _ => assert!(false)
                }
            }
            _ => assert!(false)
        }
    }

    #[test]
    fn next_turn_with_stations() {
        let (main_to_universe_sender, main_to_universe_receiver): (Sender<ExternalCommands>, Receiver<ExternalCommands>) = mpsc::channel();
        let (universe_to_main_sender, universe_to_main_receiver): (Sender<ExternalCommandReturnValues>, Receiver<ExternalCommandReturnValues>) = mpsc::channel();

        let time_stack = Communicator::new("testing".to_string());
        let channel = Channel {
            getter: main_to_universe_receiver,
            returner: universe_to_main_sender,
        };
        match time_stack.channel_sender.send(channel) {
            Err(e) => println!("Sender errored: {}", e),
            _ => {}
        }

        verify_initial_state_of_station(&main_to_universe_sender, &universe_to_main_receiver);

        check_turn(&main_to_universe_sender, &universe_to_main_receiver, 0);
        check_station_state(&main_to_universe_sender, &universe_to_main_receiver, 0, 0, 0);

        send_and_wait(&main_to_universe_sender, &universe_to_main_receiver, ExternalCommands::Time(ExternalTimeEventType::StartUntilTurn(1)));

        check_turn(&main_to_universe_sender, &universe_to_main_receiver, 1);
        check_station_state(&main_to_universe_sender, &universe_to_main_receiver, 0, 0, 0);

        send_request_to_station(&main_to_universe_sender, &universe_to_main_receiver, ExternalStationEventType::RequestLoad(LoadingRequest::new(Product::PowerCells, 200)));
        check_station_state(&main_to_universe_sender, &universe_to_main_receiver, 200, 0, 0);

        send_and_wait(&main_to_universe_sender, &universe_to_main_receiver, ExternalCommands::Time(ExternalTimeEventType::StartUntilTurn(2)));

        check_turn(&main_to_universe_sender, &universe_to_main_receiver, 2);
        check_station_state(&main_to_universe_sender, &universe_to_main_receiver, 199, 0, 1);

        send_and_wait(&main_to_universe_sender, &universe_to_main_receiver, ExternalCommands::Time(ExternalTimeEventType::StartUntilTurn(3)));

        check_turn(&main_to_universe_sender, &universe_to_main_receiver, 3);
        check_station_state(&main_to_universe_sender, &universe_to_main_receiver, 199, 2, 0);

        send_and_wait(&main_to_universe_sender, &universe_to_main_receiver, ExternalCommands::Time(ExternalTimeEventType::StartUntilTurn(10)));

        thread::sleep(Duration::from_secs(1));

        check_turn(&main_to_universe_sender, &universe_to_main_receiver, 10);
        check_station_state(&main_to_universe_sender, &universe_to_main_receiver, 195, 8, 1);

        send_request_to_station(&main_to_universe_sender, &universe_to_main_receiver, ExternalStationEventType::RequestUnload(LoadingRequest::new(Product::Ores, 8)));
        check_station_state(&main_to_universe_sender, &universe_to_main_receiver, 195, 0, 1);
    }

    fn send_request_to_station(main_to_universe_sender: &Sender<ExternalCommands>, universe_to_main_receiver: &Receiver<ExternalCommandReturnValues>, request: ExternalStationEventType) {
        match main_to_universe_sender.send(ExternalCommands::Station(request)) {
            Err(e) => println!("Sender errored: {}", e),
            _ => {}
        }

        match universe_to_main_receiver.recv_timeout(Duration::from_secs(1)).unwrap() {
            ExternalCommandReturnValues::Station(station_return) => {
                match station_return {
                    Approved => {}
                    _ => assert!(false)
                }
            }
            _ => assert!(false)
        }
    }

    fn verify_initial_state_of_station(main_to_universe_sender: &Sender<ExternalCommands>, universe_to_main_receiver: &Receiver<ExternalCommandReturnValues>) {
        match main_to_universe_sender.send(ExternalCommands::Station(ExternalStationEventType::GetStationState { include_stack: true })) {
            Err(e) => println!("Sender errored: {}", e),
            _ => {}
        }

        match universe_to_main_receiver.recv_timeout(Duration::from_secs(1)).unwrap() {
            ExternalCommandReturnValues::Station(station_return) => {
                match station_return {
                    StationState(station_state) => {
                        assert_eq!("Human ore mine", station_state.station_type());
                        assert_eq!("The digger", station_state.name());
                        assert_eq!(1, station_state.event_stack().len());

                        assert_eq!(1, station_state.production().production_time());
                        assert_eq!(0, station_state.production().production_progress());

                        assert_eq!(1, station_state.production().input().get(0).unwrap().amount());
                        assert_eq!(0, station_state.production().input().get(0).unwrap().current_storage());
                        assert_eq!(&Product::PowerCells, station_state.production().input().get(0).unwrap().product());
                        assert_eq!(10000, station_state.production().input().get(0).unwrap().max_storage());

                        assert_eq!(2, station_state.production().output().get(0).unwrap().amount());
                        assert_eq!(0, station_state.production().output().get(0).unwrap().current_storage());
                        assert_eq!(&Product::Ores, station_state.production().output().get(0).unwrap().product());
                        assert_eq!(20000, station_state.production().output().get(0).unwrap().max_storage());
                    }
                    _ => assert!(false)
                }
            }
            _ => assert!(false)
        }
    }

    fn check_station_state(main_to_universe_sender: &Sender<ExternalCommands>, universe_to_main_receiver: &Receiver<ExternalCommandReturnValues>, expected_first_input_current_storage: u32, expected_first_output_current_storage: u32, expected_production_progress: u32) {
        match main_to_universe_sender.send(ExternalCommands::Station(ExternalStationEventType::GetStationState { include_stack: true })) {
            Err(e) => println!("Sender errored: {}", e),
            _ => {}
        }

        match universe_to_main_receiver.recv_timeout(Duration::from_secs(1)).unwrap() {
            ExternalCommandReturnValues::Station(station_return) => {
                match station_return {
                    StationState(station_state) => {
                        assert_eq!(expected_first_input_current_storage, station_state.production().input().get(0).unwrap().current_storage());
                        assert_eq!(expected_first_output_current_storage, station_state.production().output().get(0).unwrap().current_storage());
                        assert_eq!(expected_production_progress, station_state.production().production_progress());
                    }
                    _ => assert!(false)
                }
            }
            _ => assert!(false)
        }
    }

    fn send_and_wait(main_to_universe_sender: &Sender<ExternalCommands>, universe_to_main_receiver: &Receiver<ExternalCommandReturnValues>, event_type: ExternalCommands) {
        match main_to_universe_sender.send(event_type) {
            Err(e) => println!("Sender errored: {}", e),
            _ => {}
        }

        await_recived(&universe_to_main_receiver);
    }

    fn check_turn(main_to_universe_sender: &Sender<ExternalCommands>, universe_to_main_receiver: &Receiver<ExternalCommandReturnValues>, expected_turn_count: u64) {
        match main_to_universe_sender.send(ExternalCommands::Time(ExternalTimeEventType::GetTimeStackState { include_stack: true })) {
            Err(e) => println!("Sender errored: {}", e),
            _ => {}
        }

        match universe_to_main_receiver.recv_timeout(Duration::from_secs(1)).unwrap() {
            ExternalCommandReturnValues::Time(time_return) => {
                match time_return {
                    StackState(state) => assert_eq!(expected_turn_count, state.turn()),
                    _ => assert!(false)
                }
            }
            _ => assert!(false)
        }
    }

    fn await_recived(universe_to_main_receiver: &Receiver<ExternalCommandReturnValues>) {
        match universe_to_main_receiver.recv_timeout(Duration::from_secs(1)).unwrap() {
            ExternalCommandReturnValues::Time(time_return) => {
                match time_return {
                    Received => {}
                    _ => assert!(false)
                }
            }
            _ => assert!(false)
        }
    }
}