use std::{thread, time};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use crate::external_commands::{ExternalCommandReturnValues, ExternalCommands};
use crate::MainConfig;
use crate::save_load::load_or_create_universe;

// channel_getter is one channel to receive new channels.
// Then the loop will listen for events from that channel to execute.
fn game_loop(channel_getter: Receiver<Channel>, config: MainConfig) {
    thread::spawn(move || {
        let mut universe = load_or_create_universe(&config);
        println!("Loaded universe with name {}", universe.universe_name());
        let mut channels = Vec::new();

        loop {
            for channel in channel_getter.try_recv() {
                channels.push(channel);
            }

            for channel in &channels {
                for event in channel.getter.try_recv() {
                    channel.returner
                        .send(universe.handle_event(event))
                        .expect("Failed sending event in gameloop.")
                }
            }

            universe.request_execute_turn();

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
    pub fn new(config: &MainConfig) -> Self {
        let (channel_sender, channel_getter): (Sender<Channel>, Receiver<Channel>) = mpsc::channel();
        let stack = Communicator { channel_sender };
        game_loop(channel_getter, config.clone());
        stack
    }
}

#[cfg(test)]
mod tests_int {
    use std::sync::mpsc;
    use std::sync::mpsc::{Receiver, Sender};
    use std::thread;
    use std::time::Duration;

    use crate::construct::construct::{ConstructEvenReturnType, ExternalConstructEventType};
    use crate::construct::construct::ConstructEvenReturnType::ConstructState;
    use crate::construct_module::ConstructModuleType;
    use crate::construct_module::ConstructModuleType::Production;
    use crate::external_commands::{Amount, ExternalCommandReturnValues, ExternalCommands};
    use crate::ExternalCommandReturnValues::Construct as ConstructEvent;
    use crate::gameloop::{Channel, Communicator};
    use crate::MainConfig;
    use crate::products::Product;
    use crate::time::ExternalTimeEventType;
    use crate::time::TimeEventReturnType::{Received, StackState};

    #[test]
    fn it_works() {
        let main_config = MainConfig {
            address : "random".to_string(),
            universe_name : "testing".to_string(),
            config_name : "default".to_string()
        };

        let (main_to_universe_sender, main_to_universe_receiver): (Sender<ExternalCommands>, Receiver<ExternalCommands>) = mpsc::channel();
        let (universe_to_main_sender, universe_to_main_receiver): (Sender<ExternalCommandReturnValues>, Receiver<ExternalCommandReturnValues>) = mpsc::channel();

        let time_stack = Communicator::new(&main_config);
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
        let main_config = MainConfig {
            address : "random".to_string(),
            universe_name : "testing".to_string(),
            config_name : "default".to_string()
        };

        let (main_to_universe_sender, main_to_universe_receiver): (Sender<ExternalCommands>, Receiver<ExternalCommands>) = mpsc::channel();
        let (universe_to_main_sender, universe_to_main_receiver): (Sender<ExternalCommandReturnValues>, Receiver<ExternalCommandReturnValues>) = mpsc::channel();

        let time_stack = Communicator::new(&main_config);
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
        let main_config = MainConfig {
            address : "random".to_string(),
            universe_name : "testing".to_string(),
            config_name : "default".to_string()
        };
        let (main_to_universe_sender, main_to_universe_receiver): (Sender<ExternalCommands>, Receiver<ExternalCommands>) = mpsc::channel();
        let (universe_to_main_sender, universe_to_main_receiver): (Sender<ExternalCommandReturnValues>, Receiver<ExternalCommandReturnValues>) = mpsc::channel();

        let time_stack = Communicator::new(&main_config);
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
    fn next_turn_with_constructs() {
        let main_config = MainConfig {
            address : "random".to_string(),
            universe_name : "testing".to_string(),
            config_name : "default".to_string()
        };
        let (main_to_universe_sender, main_to_universe_receiver): (Sender<ExternalCommands>, Receiver<ExternalCommands>) = mpsc::channel();
        let (universe_to_main_sender, universe_to_main_receiver): (Sender<ExternalCommandReturnValues>, Receiver<ExternalCommandReturnValues>) = mpsc::channel();

        let time_stack = Communicator::new(&main_config);
        let channel = Channel {
            getter: main_to_universe_receiver,
            returner: universe_to_main_sender,
        };
        match time_stack.channel_sender.send(channel) {
            Err(e) => println!("Sender errored: {}", e),
            _ => {}
        }

        verify_initial_state_of_construct(&main_to_universe_sender, &universe_to_main_receiver);

        check_turn(&main_to_universe_sender, &universe_to_main_receiver, 0);
        check_construct_state(&main_to_universe_sender, &universe_to_main_receiver, None, None, 0);

        send_and_wait(&main_to_universe_sender, &universe_to_main_receiver, ExternalCommands::Time(ExternalTimeEventType::StartUntilTurn(1)));

        check_turn(&main_to_universe_sender, &universe_to_main_receiver, 1);
        check_construct_state(&main_to_universe_sender, &universe_to_main_receiver, None, None, 0);

        send_load_request_to_construct(&main_to_universe_sender, &universe_to_main_receiver, Amount::new(Product::PowerCells, 200));
        check_construct_state(&main_to_universe_sender, &universe_to_main_receiver, Some(&200), None, 0);

        send_and_wait(&main_to_universe_sender, &universe_to_main_receiver, ExternalCommands::Time(ExternalTimeEventType::StartUntilTurn(2)));

        check_turn(&main_to_universe_sender, &universe_to_main_receiver, 2);
        check_construct_state(&main_to_universe_sender, &universe_to_main_receiver, Some(&199), None, 3);

        send_and_wait(&main_to_universe_sender, &universe_to_main_receiver, ExternalCommands::Time(ExternalTimeEventType::StartUntilTurn(3)));

        check_turn(&main_to_universe_sender, &universe_to_main_receiver, 3);
        check_construct_state(&main_to_universe_sender, &universe_to_main_receiver, Some(&198), Some(&2), 4);

        send_and_wait(&main_to_universe_sender, &universe_to_main_receiver, ExternalCommands::Time(ExternalTimeEventType::StartUntilTurn(10)));

        thread::sleep(Duration::from_secs(1));

        check_turn(&main_to_universe_sender, &universe_to_main_receiver, 10);
        check_construct_state(&main_to_universe_sender, &universe_to_main_receiver, Some(&191), Some(&16), 11);

        send_unload_request_to_construct(&main_to_universe_sender, &universe_to_main_receiver, Amount::new(Product::Ores, 8));
        check_construct_state(&main_to_universe_sender, &universe_to_main_receiver, Some(&191), Some(&8), 11);
    }

    fn send_load_request_to_construct(main_to_universe_sender: &Sender<ExternalCommands>, universe_to_main_receiver: &Receiver<ExternalCommandReturnValues>, request: Amount) {
        match main_to_universe_sender.send(ExternalCommands::Construct("The_base".to_string(), ExternalConstructEventType::RequestLoad(request))) {
            Err(e) => println!("Sender errored: {}", e),
            _ => {}
        }

        match universe_to_main_receiver.recv_timeout(Duration::from_secs(1)).unwrap() {
            ConstructEvent(station_return) => {
                match station_return {
                    ConstructEvenReturnType::RequestLoadProcessed(_) => {}
                    _ => assert!(false)
                }
            }
            _ => assert!(false)
        }
    }

    fn send_unload_request_to_construct(main_to_universe_sender: &Sender<ExternalCommands>, universe_to_main_receiver: &Receiver<ExternalCommandReturnValues>, request: Amount) {
        match main_to_universe_sender.send(ExternalCommands::Construct("The_base".to_string(), ExternalConstructEventType::RequestUnload(request))) {
            Err(e) => println!("Sender errored: {}", e),
            _ => {}
        }

        match universe_to_main_receiver.recv_timeout(Duration::from_secs(1)).unwrap() {
            ExternalCommandReturnValues::Construct(construct_return) => {
                match construct_return {
                    ConstructEvenReturnType::RequestUnloadProcessed(_) => {}
                    _ => assert!(false)
                }
            }
            _ => assert!(false)
        }
    }

    fn verify_initial_state_of_construct(main_to_universe_sender: &Sender<ExternalCommands>, universe_to_main_receiver: &Receiver<ExternalCommandReturnValues>) {
        match main_to_universe_sender.send(ExternalCommands::Construct("The_base".to_string(), ExternalConstructEventType::GetConstructState { include_stack: true })) {
            Err(e) => println!("Sender errored: {}", e),
            _ => {}
        }

        match universe_to_main_receiver.recv_timeout(Duration::from_secs(1)).unwrap() {
            ConstructEvent(station_return) => {
                match station_return {
                    ConstructState(construct_state) => {
                        assert_eq!("The_base", construct_state.name());
                        //assert_eq!(1, construct_state.event_stack().len());

                        assert_eq!(500, construct_state.capacity());
                        assert_eq!(1, construct_state.modules().len());
                        assert_eq!(0, construct_state.current_storage().values().sum::<u32>());

                        let production = construct_state.modules().iter()
                            .filter_map(|p|
                                match p {
                                    ConstructModuleType::Production(production) => {
                                        if production.name().eq("PowerToOre") {
                                            Some(production)
                                        } else {
                                            None
                                        }
                                    }
                                })
                            .next()
                            .unwrap();


                        assert_eq!(0, production.production_trigger_time());
                        assert_eq!(1, production.production_time());
                        assert_eq!(false, production.stored_output());
                        assert_eq!(false, production.stored_input());
                        assert_eq!(&vec![Amount::new(Product::PowerCells, 1); 1], production.input());
                        assert_eq!(&vec![Amount::new(Product::Ores, 2); 1], production.output());
                    }
                    _ => assert!(false)
                }
            }
            _ => assert!(false)
        }
    }

    fn check_construct_state(main_to_universe_sender: &Sender<ExternalCommands>, universe_to_main_receiver: &Receiver<ExternalCommandReturnValues>, expected_first_input_current_storage: Option<&u32>, expected_first_output_current_storage: Option<&u32>, production_trigger_time: u64) {
        match main_to_universe_sender.send(ExternalCommands::Construct("The_base".to_string(), ExternalConstructEventType::GetConstructState { include_stack: true })) {
            Err(e) => println!("Sender errored: {}", e),
            _ => {}
        }

        match universe_to_main_receiver.recv_timeout(Duration::from_secs(1)).unwrap() {
            ConstructEvent(construct_return) => {
                match construct_return {
                    ConstructState(construct_state) => {
                        assert_eq!(expected_first_input_current_storage, construct_state.current_storage().get(&Product::PowerCells));
                        assert_eq!(expected_first_output_current_storage, construct_state.current_storage().get(&Product::Ores));
                        assert_eq!(production_trigger_time, if let Some(Production(production_module)) = construct_state.modules().get(0) { production_module.production_trigger_time() } else { unreachable!() });
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