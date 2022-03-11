
use crate::products::Product;

#[derive(Clone, PartialEq, Debug)]
pub struct Amount {
    product: Product,
    amount: u32,
    current_storage: u32,
    max_storage: u32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Production {
    input: Vec<Amount>,
    output: Vec<Amount>,
    production_time: u32,
    production_progress: u32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct StationState {
    name: String,
    station_type: String,
    production: Production,
    stack: Vec<StationEvenType>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct LoadingRequest {
    product: Product,
    amount: u32,
}

#[derive(Clone, PartialEq, Debug)]
pub enum StationEvenType {
    RequestLoad(LoadingRequest),
    RequestUnload(LoadingRequest),
    GetStationState,
    ExecuteTurn,
}

#[derive(Clone, PartialEq, Debug)]
pub enum StationEvenReturnType {
    Denied(String),
    Approved,
    StationState(StationState),
    TurnExecuted,
}

pub fn push_event(station_state: &mut StationState, event: &StationEvenType) -> StationEvenReturnType {
    station_state.stack.push(event.clone());
    handle_event(station_state, event)
}

fn handle_event(station_state: &mut StationState, event: &StationEvenType) -> StationEvenReturnType {
    return match event {
        StationEvenType::GetStationState => {
            StationEvenReturnType::StationState(station_state.clone())
        }
        StationEvenType::RequestLoad(request) => {
            for input in &mut station_state.production.input {
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
        StationEvenType::RequestUnload(request) => {
            for output in &mut station_state.production.output {
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
        StationEvenType::ExecuteTurn => {
            next_turn(station_state);
            StationEvenReturnType::TurnExecuted
        }
    };
}

fn next_turn(station_state: &mut StationState) {
    let production_progress = station_state.production.production_progress;

    if production_progress == 0 {
        if have_all_inputs(&station_state) {
            station_state.production.production_progress += 1;
            subtract_all_inputs(station_state);
        }
    } else if production_progress == station_state.production.production_time {
        if have_room_for_outputs(&station_state) {
            station_state.production.production_progress = 0;
            add_all_outputs(station_state);
        }
    } else {
        station_state.production.production_progress += 1;
    }
}


fn have_all_inputs(station_state: &StationState) -> bool {
    for input in &station_state.production.input {
        if input.current_storage < input.amount {
            return false;
        }
    }
    return true;
}

fn subtract_all_inputs(station_state: &mut StationState) {
    for mut input in &mut station_state.production.input {
        input.current_storage -= input.amount;
    }
}

fn have_room_for_outputs(station_state: &StationState) -> bool {
    for output in &station_state.production.output {
        if output.current_storage + output.amount > output.max_storage {
            return false;
        }
    }
    return true;
}

fn add_all_outputs(station_state: &mut StationState) {
    for mut output in &mut station_state.production.output {
        output.current_storage += output.amount;
    }
}

#[cfg(test)]
mod tests_int {
    use crate::products::Product;
    use crate::station::{Amount, push_event, LoadingRequest, Production, StationEvenReturnType, StationEvenType, StationState};

    #[test]
    fn request_unload_wrong_product() {
        let mut station = make_mining_station();
        match push_event(&mut station, &StationEvenType::RequestUnload(LoadingRequest {
            product: Product::Metals,
            amount: 200,
        })) {
            StationEvenReturnType::Denied(s) => { assert_eq!("Unloading request denied. This station does not produce Metals and will not sell it.", s) }
            _ => assert!(false)
        }
    }

    #[test]
    fn request_unload_to_big_amount() {
        let mut station = make_mining_station();
        match push_event(&mut station, &StationEvenType::RequestUnload(LoadingRequest {
            product: Product::Ores,
            amount: 200,
        })) {
            StationEvenReturnType::Denied(s) => { assert_eq!("Unloading request denied. Requested 200 but there were only 0 available.", s) }
            _ => assert!(false)
        }
    }

    #[test]
    fn request_load_wrong_product() {
        let mut station = make_mining_station();
        match push_event(&mut station, &StationEvenType::RequestLoad(LoadingRequest {
            product: Product::Ores,
            amount: 200,
        })) {
            StationEvenReturnType::Denied(s) => { assert_eq!("Loading request denied. This station does not use Ores and will not receive it.", s) }
            _ => assert!(false)
        }
    }

    #[test]
    fn request_load_wrong_amount() {
        let mut station = make_mining_station();
        match push_event(&mut station, &StationEvenType::RequestLoad(LoadingRequest {
            product: Product::PowerCells,
            amount: 9999999,
        })) {
            StationEvenReturnType::Denied(s) => { assert_eq!("Loading request denied. Requested 9999999 but there were only room for 10000.", s) }
            _ => assert!(false)
        }
    }

    #[test]
    fn produce() {
        let mut station = make_mining_station();
        match push_event(&mut station, &StationEvenType::RequestLoad(LoadingRequest {
            product: Product::PowerCells,
            amount: 100,
        })) {
            StationEvenReturnType::Approved => {}
            _ => assert!(false)
        }

        match push_event(&mut station, &StationEvenType::ExecuteTurn) {
            StationEvenReturnType::TurnExecuted => {}
            _ => assert!(false)
        }

        match push_event(&mut station, &StationEvenType::GetStationState) {
            StationEvenReturnType::StationState(state) => {
                assert_eq!(0, state.production.output.get(0).unwrap().current_storage);
                assert_eq!(99, state.production.input.get(0).unwrap().current_storage);
                assert_eq!(1, state.production.production_progress);
            }
            _ => assert!(false)
        }

        match push_event(&mut station, &StationEvenType::ExecuteTurn) {
            StationEvenReturnType::TurnExecuted => {}
            _ => assert!(false)
        }

        match push_event(&mut station, &StationEvenType::GetStationState) {
            StationEvenReturnType::StationState(state) => {
                assert_eq!(2, state.production.output.get(0).unwrap().current_storage);
                assert_eq!(99, state.production.input.get(0).unwrap().current_storage);
                assert_eq!(0, state.production.production_progress);
            }
            _ => assert!(false)
        }

        match push_event(&mut station, &StationEvenType::ExecuteTurn) {
            StationEvenReturnType::TurnExecuted => {}
            _ => assert!(false)
        }

        match push_event(&mut station, &StationEvenType::GetStationState) {
            StationEvenReturnType::StationState(state) => {
                assert_eq!(2, state.production.output.get(0).unwrap().current_storage);
                assert_eq!(98, state.production.input.get(0).unwrap().current_storage);
                assert_eq!(1, state.production.production_progress);
            }
            _ => assert!(false)
        }

        match push_event(&mut station, &StationEvenType::RequestUnload(LoadingRequest {
            product: Product::Ores,
            amount: 2,
        })) {
            StationEvenReturnType::Approved => {}
            _ => assert!(false)
        }

        match push_event(&mut station, &StationEvenType::GetStationState) {
            StationEvenReturnType::StationState(state) => {
                assert_eq!(0, state.production.output.get(0).unwrap().current_storage);
                assert_eq!(98, state.production.input.get(0).unwrap().current_storage);
                assert_eq!(1, state.production.production_progress);
            }
            _ => assert!(false)
        }
    }

    fn make_mining_station() -> StationState {
        let energy_cells_amount = vec![Amount {
            product: Product::PowerCells,
            amount: 1,
            current_storage: 0,
            max_storage: 10000,
        }];
        let ore_amount = vec![Amount {
            product: Product::Ores,
            amount: 2,
            current_storage: 0,
            max_storage: 20000,
        }];

        StationState {
            name: "The digger".to_string(),
            station_type: "Human ore mine".to_string(),
            production: Production
            {
                input: energy_cells_amount,
                output: ore_amount,
                production_time: 1,
                production_progress: 0,
            },
            stack: Vec::new(),
        }
    }
}