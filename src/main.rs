use std::{env, fs};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::ops::Add;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::external_commands::{ExternalCommandReturnValues, ExternalCommands};
use crate::gameloop::{Channel, Communicator};

mod time;
mod gameloop;
mod products;
mod station;
mod external_commands;
mod save_load;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct MainConfig {
    address: String,
}

fn main() {
    let (main_config, universe_name) = read_main_config_file();

    let (listener, main_to_universe_sender, universe_to_main_receiver) = setup_game(&main_config, universe_name);

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                handle_request(&main_to_universe_sender, &universe_to_main_receiver, &mut stream)
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }

    // close the socket server
    drop(listener);
}

fn read_main_config_file() -> (MainConfig, String) {
    let args: Vec<String> = env::args().collect();

    let universe_name = if args.len() > 1 {
        &args[1]
    } else {
        panic!("First argument of starting the game is the universe_name, got: {:?}", args)
    };

    let config_name = if args.len() > 2 {
        &args[2]
    } else {
        "default/"
    };

    let config_folder = "./config/".to_string().add(config_name);
    println!("Using main config folder: {}", config_folder);

    let main_config_path = config_folder.to_string().add("main.json");

    let main_setup_config = fs::read_to_string(main_config_path)
        .expect("Something went wrong reading the file");

    let main_config: MainConfig = serde_json::from_str(main_setup_config.as_str()).unwrap();
    (main_config, universe_name.clone())
}

fn handle_request(main_to_universe_sender: &Sender<ExternalCommands>, universe_to_main_receiver: &Receiver<ExternalCommandReturnValues>, stream: &mut TcpStream) {
    // connection succeeded
    let mut buffer = [0; 1024];
    if let Err(e) = stream.set_read_timeout(Some(Duration::from_secs(1))) {
        println!("Got error from setting timeout on reading tcp input, aborting: {}", e);
        return;
    }
    let buffer_size = match stream.read(&mut buffer) {
        Ok(buffer_size_value) => { buffer_size_value }
        Err(e) => {
            println!("Got error from reading command, aborting: {}", e);
            return;
        }
    };
    let command = &buffer[..buffer_size];
    let command_as_string = String::from_utf8(command.to_vec()).unwrap();
    println!("Received request with following command: {}", command_as_string);

    match ExternalCommands::try_from(&command_as_string) {
        Err(e) => {
            if let Err(e) = stream.write(format!("{:?}", e).as_bytes()) {
                panic!("{}", e);
            }
        }
        Ok(command_enum) => {
            match main_to_universe_sender.send(command_enum) {
                Err(e) => {
                    if let Err(e) = stream.write(format!("Sender errored: {}", e).as_bytes()) {
                        panic!("{}", e);
                    }
                }
                _ => {}
            }

            let return_values = universe_to_main_receiver.recv_timeout(Duration::from_secs(1)).unwrap();

            if let Err(e) = stream.write(format!("{} \n", json!(return_values)).as_bytes()) {
                panic!("{}", e);
            }
        }
    }

    println!("Handled request with following command: {}", command_as_string);
}

fn setup_game(config: &MainConfig, universe_name: String) -> (TcpListener, Sender<ExternalCommands>, Receiver<ExternalCommandReturnValues>) {
    let listener = TcpListener::bind(&config.address).unwrap();

    let communicator = Communicator::new(universe_name);
    let (main_to_universe_sender, main_to_universe_receiver): (Sender<ExternalCommands>, Receiver<ExternalCommands>) = mpsc::channel();
    let (universe_to_main_sender, universe_to_main_receiver): (Sender<ExternalCommandReturnValues>, Receiver<ExternalCommandReturnValues>) = mpsc::channel();

    let channel = Channel::new(main_to_universe_receiver, universe_to_main_sender);

    match communicator.channel_sender().send(channel) {
        Err(e) => println!("Sender errored: {}", e),
        _ => {}
    }
    println!("Game is ready and listening on: {}", &config.address);
    (listener, main_to_universe_sender, universe_to_main_receiver)
}
