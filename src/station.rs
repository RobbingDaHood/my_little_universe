use std::io::SeekFrom;
use serde::{Deserialize, Serialize};
use crate::construct_module::CanHandleNextTurn;

use crate::products::Product;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct LoadingRequest {
    product: Product,
    amount: u32,
}

impl LoadingRequest {
    pub fn new(product: Product, amount: u32) -> Self {
        LoadingRequest { product, amount }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum StationEventType {
    Internal(InternalStationEventType),
    External(ExternalStationEventType),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum InternalStationEventType {
    ExecuteTurn(u64),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ExternalStationEventType {
    RequestLoad(LoadingRequest),
    RequestUnload(LoadingRequest),
    GetStationState { include_stack: bool },
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum StationEvenReturnType {
    Denied(String),
    Approved,
    StationState(Station),
    TurnExecuted,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Amount {
    product: Product,
    amount: u32,
    current_storage: u32,
    max_storage: u32,
}

impl Amount {
    pub fn product(&self) -> &Product {
        &self.product
    }
    pub fn amount(&self) -> u32 {
        self.amount
    }
    pub fn current_storage(&self) -> u32 {
        self.current_storage
    }
    pub fn max_storage(&self) -> u32 {
        self.max_storage
    }
    pub fn new(product: Product, amount: u32, current_storage: u32, max_storage: u32) -> Self {
        Self { product, amount, current_storage, max_storage }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Production {
    input: Vec<Amount>,
    output: Vec<Amount>,
    production_time: u32,
    production_trigger_time: u64,
}

impl Production {
    pub fn input(&self) -> &Vec<Amount> {
        &self.input
    }
    pub fn output(&self) -> &Vec<Amount> {
        &self.output
    }
    pub fn production_time(&self) -> u32 {
        self.production_time
    }
    pub fn production_trigger_time(&self) -> u64 {
        self.production_trigger_time
    }

    pub fn new(input: Vec<Amount>, output: Vec<Amount>, production_time: u32, production_trigger_time: u64) -> Self {
        Self { input, output, production_time, production_trigger_time }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Station {
    name: String,
    production: Production,
    event_stack: Vec<StationEventType>,
}

impl Station {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn production(&self) -> &Production {
        &self.production
    }
    pub fn event_stack(&self) -> &Vec<StationEventType> {
        &self.event_stack
    }
    pub fn new(name: String, production: Production) -> Self {
        Station {
            name,
            production,
            event_stack: Vec::new(),
        }
    }

    pub fn push_event(&mut self, event: &StationEventType) -> StationEvenReturnType {
        self.event_stack.push(event.clone());
        self.handle_event(event)
    }

    fn handle_event(&mut self, event: &StationEventType) -> StationEvenReturnType {
        return match event {
            StationEventType::External(ExternalStationEventType::GetStationState { include_stack }) => {
                if *include_stack {
                    StationEvenReturnType::StationState(self.clone())
                } else {
                    let mut state = self.clone();
                    state.event_stack = Vec::new();
                    StationEvenReturnType::StationState(state)
                }
            }
            StationEventType::External(ExternalStationEventType::RequestLoad(request)) => {
                for input in &mut self.production.input {
                    if input.product == request.product {
                        match input.current_storage.checked_add(request.amount) {
                            Some(new_current_amount) => {
                                if new_current_amount <= input.max_storage {
                                    input.current_storage = new_current_amount;
                                    return StationEvenReturnType::Approved;
                                }
                            }
                            None => {}
                        };
                        return StationEvenReturnType::Denied(format!("Loading request denied. Requested {} but there were only room for {}.", request.amount, input.max_storage - input.current_storage));
                    }
                }
                StationEvenReturnType::Denied(format!("Loading request denied. This station does not use {:?} and will not receive it.", &request.product))
            }
            StationEventType::External(ExternalStationEventType::RequestUnload(request)) => {
                for output in &mut self.production.output {
                    if output.product == request.product {
                        return match output.current_storage.checked_sub(request.amount) {
                            Some(new_current_storage) => {
                                output.current_storage = new_current_storage;
                                StationEvenReturnType::Approved
                            }
                            None => {
                                StationEvenReturnType::Denied(format!("Unloading request denied. Requested {} but there were only {} available.", &request.amount, output.current_storage))
                            }
                        };
                    }
                }
                StationEvenReturnType::Denied(format!("Unloading request denied. This station does not produce {:?} and will not sell it.", &request.product))
            }
            StationEventType::Internal(InternalStationEventType::ExecuteTurn(current_turn)) => {
                self.next_turn(&current_turn);
                StationEvenReturnType::TurnExecuted
            }
        };
    }

    fn have_all_inputs(&mut self) -> bool {
        for input in &self.production.input {
            if input.current_storage < input.amount {
                return false;
            }
        }
        return true;
    }

    fn subtract_all_inputs(&mut self) {
        for mut input in &mut self.production.input {
            input.current_storage -= input.amount;
        }
    }

    fn have_room_for_outputs(&mut self) -> bool {
        for output in &self.production.output {
            if output.current_storage + output.amount > output.max_storage {
                return false;
            }
        }
        return true;
    }

    fn add_all_outputs(&mut self) {
        for mut output in &mut self.production.output {
            output.current_storage += output.amount;
        }
    }
}

impl CanHandleNextTurn for Station {
    fn next_turn(&mut self, current_turn: &u64) {
        if current_turn >= &self.production.production_trigger_time {
            if self.have_all_inputs() {
                self.subtract_all_inputs();
                self.production.production_trigger_time = current_turn + self.production.production_time as u64;
            }
            if self.production.production_trigger_time > 0 && self.have_room_for_outputs() {
                self.add_all_outputs();
            }
        }
    }
}

#[cfg(test)]
mod tests_int {
    use serde_json::json;

    use crate::products::Product;
    use crate::station::{Amount, ExternalStationEventType, InternalStationEventType, LoadingRequest, Production, StationEvenReturnType, StationEventType, Station};

    #[test]
    fn request_unload_wrong_product() {
        let mut station = make_mining_station();
        match station.push_event(&StationEventType::External(ExternalStationEventType::RequestUnload(LoadingRequest {
            product: Product::Metals,
            amount: 200,
        }))) {
            StationEvenReturnType::Denied(s) => { assert_eq!("Unloading request denied. This station does not produce Metals and will not sell it.", s) }
            _ => assert!(false)
        }
    }

    #[test]
    fn request_unload_to_big_amount() {
        let mut station = make_mining_station();
        match station.push_event(&StationEventType::External(ExternalStationEventType::RequestUnload(LoadingRequest {
            product: Product::Ores,
            amount: 200,
        }))) {
            StationEvenReturnType::Denied(s) => { assert_eq!("Unloading request denied. Requested 200 but there were only 0 available.", s) }
            _ => assert!(false)
        }
    }

    #[test]
    fn request_load_wrong_product() {
        let mut station = make_mining_station();
        match station.push_event(&StationEventType::External(ExternalStationEventType::RequestLoad(LoadingRequest {
            product: Product::Ores,
            amount: 200,
        }))) {
            StationEvenReturnType::Denied(s) => { assert_eq!("Loading request denied. This station does not use Ores and will not receive it.", s) }
            _ => assert!(false)
        }
    }

    #[test]
    fn request_load_wrong_amount() {
        let mut station = make_mining_station();
        match station.push_event(&StationEventType::External(ExternalStationEventType::RequestLoad(LoadingRequest {
            product: Product::PowerCells,
            amount: 9999999,
        }))) {
            StationEvenReturnType::Denied(s) => { assert_eq!("Loading request denied. Requested 9999999 but there were only room for 10000.", s) }
            _ => assert!(false)
        }
    }

    #[test]
    fn produce() {
        let mut station = make_mining_station();
        match station.push_event(&StationEventType::External(ExternalStationEventType::RequestLoad(LoadingRequest {
            product: Product::PowerCells,
            amount: 100,
        }))) {
            StationEvenReturnType::Approved => {}
            _ => assert!(false)
        }

        match station.push_event(&StationEventType::Internal(InternalStationEventType::ExecuteTurn(1))) {
            StationEvenReturnType::TurnExecuted => {}
            _ => assert!(false)
        }

        match station.push_event(&StationEventType::External(ExternalStationEventType::GetStationState { include_stack: true })) {
            StationEvenReturnType::StationState(state) => {
                assert_eq!(2, state.production.output.get(0).unwrap().current_storage);
                assert_eq!(99, state.production.input.get(0).unwrap().current_storage);
                assert_eq!(2, state.production.production_trigger_time);
            }
            _ => assert!(false)
        }

        match station.push_event(&StationEventType::Internal(InternalStationEventType::ExecuteTurn(2))) {
            StationEvenReturnType::TurnExecuted => {}
            _ => assert!(false)
        }

        match station.push_event(&StationEventType::External(ExternalStationEventType::GetStationState { include_stack: true })) {
            StationEvenReturnType::StationState(state) => {
                assert_eq!(4, state.production.output.get(0).unwrap().current_storage);
                assert_eq!(98, state.production.input.get(0).unwrap().current_storage);
                assert_eq!(3, state.production.production_trigger_time);
            }
            _ => assert!(false)
        }

        match station.push_event(&StationEventType::External(ExternalStationEventType::RequestUnload(LoadingRequest {
            product: Product::Ores,
            amount: 2,
        }))) {
            StationEvenReturnType::Approved => {}
            _ => assert!(false)
        }

        match station.push_event(&StationEventType::External(ExternalStationEventType::GetStationState { include_stack: true })) {
            StationEvenReturnType::StationState(state) => {
                assert_eq!(2, state.production.output.get(0).unwrap().current_storage);
                assert_eq!(98, state.production.input.get(0).unwrap().current_storage);
                assert_eq!(3, state.production.production_trigger_time);
            }
            _ => assert!(false)
        }
    }

    #[test]
    fn serialise_deserialize() {
        let mut station = make_mining_station();
        execute_all_events(&mut station);
        let json = json!(station).to_string();
        let and_back = serde_json::from_str(&json).unwrap();
        assert_eq!(station, and_back);
    }

    fn execute_all_events(station: &mut Station) {
        match station.push_event(&StationEventType::External(ExternalStationEventType::RequestLoad(LoadingRequest {
            product: Product::PowerCells,
            amount: 2,
        }))) {
            StationEvenReturnType::Approved => {}
            _ => assert!(false)
        }
        match station.push_event(&StationEventType::External(ExternalStationEventType::GetStationState { include_stack: true })) {
            StationEvenReturnType::StationState(_) => {}
            _ => assert!(false)
        }
        match station.push_event(&StationEventType::Internal(InternalStationEventType::ExecuteTurn(0))) {
            StationEvenReturnType::TurnExecuted => {}
            _ => assert!(false)
        }
        match station.push_event(&StationEventType::Internal(InternalStationEventType::ExecuteTurn(0))) {
            StationEvenReturnType::TurnExecuted => {}
            _ => assert!(false)
        }
        match station.push_event(&StationEventType::External(ExternalStationEventType::RequestUnload(LoadingRequest {
            product: Product::Ores,
            amount: 2,
        }))) {
            StationEvenReturnType::Approved => {}
            _ => assert!(false)
        }
    }

    fn make_mining_station() -> Station {
        Station {
            name: "The digger".to_string(),
            production: Production
            {
                input: vec![Amount {
                    product: Product::PowerCells,
                    amount: 1,
                    current_storage: 0,
                    max_storage: 10000,
                }],
                output: vec![Amount {
                    product: Product::Ores,
                    amount: 2,
                    current_storage: 0,
                    max_storage: 20000,
                }],
                production_time: 1,
                production_trigger_time: 0,
            },
            event_stack: Vec::new(),
        }
    }
}