use std::cmp::min;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::construct::amount::Amount;
use crate::construct::construct::ConstructEvenReturnType::{RequestLoadProcessed, RequestUnloadProcessed};
use crate::construct::construct_position::{ConstructPositionEventReturnType, ConstructPositionEventType, ConstructPositionSector, ConstructPositionState, ExternalConstructPositionEventType, InternalConstructPositionEventType};
use crate::construct::production_module::ProductionModule;
use crate::construct_module::{CanHandleNextTurn, ConstructModuleType};
use crate::products::Product;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ConstructEventType {
    Internal(InternalConstructEventType),
    External(ExternalConstructEventType),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum InternalConstructEventType {
    ExecuteTurn(u64),
    ConstructPosition(InternalConstructPositionEventType),
    RequestLoad(Amount),
    RequestUnload(Amount),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ExternalConstructEventType {
    GetConstructState { include_stack: bool },
    ConstructPosition(ExternalConstructPositionEventType),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ConstructEvenReturnType {
    RequestLoadProcessed(u32),
    RequestUnloadProcessed(u32),
    ConstructState(Construct),
    TurnExecuted,
    ConstructPosition(ConstructPositionEventReturnType),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Construct {
    name: String,
    capacity: u32,
    current_storage: HashMap<Product, u32>,
    modules: Vec<ConstructModuleType>,
    event_stack: Vec<ConstructEventType>,
    pub(crate) position: ConstructPositionState,
}

impl Construct {
    pub fn new(name: String, capacity: u32, sector_position: ConstructPositionSector) -> Self {
        Construct { name: name.clone(), capacity, current_storage: HashMap::new(), modules: Vec::new(), event_stack: Vec::new(), position: ConstructPositionState::new(sector_position) }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn capacity(&self) -> u32 {
        self.capacity
    }
    pub fn current_storage(&self) -> &HashMap<Product, u32> {
        &self.current_storage
    }
    pub fn modules(&self) -> &Vec<ConstructModuleType> {
        &self.modules
    }
    pub fn event_stack(&self) -> &Vec<ConstructEventType> {
        &self.event_stack
    }
    pub fn position(&self) -> &ConstructPositionState {
        &self.position
    }

    pub fn push_event(&mut self, event: &ConstructEventType) -> ConstructEvenReturnType {
        // self.event_stack.push(event.clone());
        self.handle_event(event)
    }

    fn handle_event(&mut self, event: &ConstructEventType) -> ConstructEvenReturnType {
        return match event {
            ConstructEventType::External(ExternalConstructEventType::GetConstructState { include_stack }) => {
                if *include_stack {
                    ConstructEvenReturnType::ConstructState(self.clone())
                } else {
                    let mut state = self.clone();
                    state.event_stack = Vec::new();
                    ConstructEvenReturnType::ConstructState(state)
                }
            }
            ConstructEventType::Internal(InternalConstructEventType::RequestLoad(request)) => {
                RequestLoadProcessed(self.load_request(request))
            }
            ConstructEventType::Internal(InternalConstructEventType::RequestUnload(request)) => {
                RequestUnloadProcessed(self.unload_request(request))
            }
            ConstructEventType::External(ExternalConstructEventType::ConstructPosition(construct_position_event)) => {
                match construct_position_event {
                    ExternalConstructPositionEventType::Dock(target_construct) => {
                        if self.name.eq(target_construct) {
                            return ConstructEvenReturnType::ConstructPosition(ConstructPositionEventReturnType::Denied("Construct cannot dock with itself.".to_string()));
                        }
                    }
                    _ => {}
                }
                ConstructEvenReturnType::ConstructPosition(self.position.handle_event(&ConstructPositionEventType::External(construct_position_event.clone())))
            }
            ConstructEventType::Internal(InternalConstructEventType::ExecuteTurn(current_turn)) => {
                self.next_turn(&current_turn);
                ConstructEvenReturnType::TurnExecuted
            }
            ConstructEventType::Internal(InternalConstructEventType::ConstructPosition(construct_position_event)) => {
                ConstructEvenReturnType::ConstructPosition(self.position.handle_event(&ConstructPositionEventType::Internal(construct_position_event.clone())))
            }
        };
    }

    pub fn unload_request(&mut self, amount: &Amount) -> u32 {
        unload(&mut self.current_storage, amount)
    }

    pub(crate) fn load_request(&mut self, amount: &Amount) -> u32 {
        let leftover_capacity = self.capacity - self.current_storage.values().sum::<u32>();
        let amount_to_be_stored = min(leftover_capacity, amount.amount());

        if amount_to_be_stored == 0 {
            return amount.amount();
        }

        load(&mut self.current_storage, &Amount::new(amount.product().clone(), amount_to_be_stored));

        amount_to_be_stored
    }

    pub fn install(&mut self, new_module: ConstructModuleType) -> Result<(), String> {
        if self.modules.iter()
            .find(|m| m.name().eq(new_module.name()))
            .is_some() {
            return Err("Module with that name already exists.".to_string());
        }

        self.modules.push(new_module);
        Ok(())
    }

    fn uninstall(&mut self, module_name: &String) -> Result<(), String> {
        if self.modules.iter()
            .find(|m| m.name().eq(module_name))
            .is_none() {
            return Err("Module with that name is not installed.".to_string());
        }

        self.modules.retain(|m| !m.name().eq(module_name));
        Ok(())
    }
}

impl CanHandleNextTurn for Construct {
    fn next_turn(&mut self, current_turn: &u64) {
        for module in &mut self.modules {
            match module {
                ConstructModuleType::Production(production_module) => {
                    handle_production_output(&mut self.current_storage, self.capacity, current_turn, production_module);
                    handle_production_input(&mut self.current_storage, current_turn, production_module);
                    production_module.handle_turn(current_turn);
                }
            }
        }
    }
}

fn load(current_storage: &mut HashMap<Product, u32>, amount: &Amount) {
    match current_storage.get_mut(amount.product()) {
        Some(amount_stored) => {
            *amount_stored += amount.amount();
        }
        None => {
            current_storage.insert(amount.product().clone(), amount.amount());
        }
    }
}

fn unload(current_storage: &mut HashMap<Product, u32>, amount: &Amount) -> u32 {
    match current_storage.get_mut(amount.product()) {
        Some(amount_stored) => {
            if *amount_stored > amount.amount() {
                *amount_stored -= amount.amount();
                amount.amount()
            } else {
                return match current_storage.remove(amount.product()) {
                    Some(amount_stored) => { min(amount_stored, amount.amount()) }
                    None => { panic!("Just checked that product were there, but now it is not. Concurrency issue. Good luck! ") }
                };
            }
        }
        None => {
            0
        }
    }
}

fn handle_production_output(current_storage: &mut HashMap<Product, u32>, capacity: u32, current_turn: &u64, production_module: &mut ProductionModule) {
    if let Some(amounts) = production_module.will_output(current_turn) {
        let total_output = amounts.iter()
            .map(|amount| amount.amount())
            .sum::<u32>();
        let total_free_capacity = capacity - current_storage.values().sum::<u32>();

        if total_output <= total_free_capacity {
            for amount in amounts {
                load(current_storage, amount);
            }
            production_module.set_stored_output(false);
            production_module.set_stored_input(false);
        } else {
            production_module.set_stored_output(true);
        }
    }
}

fn handle_production_input(current_storage: &mut HashMap<Product, u32>, current_turn: &u64, production_module: &mut ProductionModule) {
    if let Some(amounts) = production_module.require_input(current_turn) {
        let any_product_not_stored = amounts.iter()
            .find(|amount| current_storage.get(amount.product()) == None
                || current_storage.get(amount.product()).unwrap() < &amount.amount());

        if any_product_not_stored == None {
            for amount in amounts {
                unload(current_storage, amount);
            }
            production_module.set_stored_input(true);
        }
    }
}

#[cfg(test)]
mod tests_int {
    use crate::construct::amount::Amount;
    use crate::construct::construct::{Construct, ConstructEvenReturnType, ConstructEventType, ExternalConstructEventType, InternalConstructEventType};
    use crate::construct::construct_position::{ConstructPositionEventReturnType, ConstructPositionSector, ExternalConstructPositionEventType, InternalConstructPositionEventType};
    use crate::construct::construct_position::ConstructPositionStatus::{InSector, IsDocked};
    use crate::construct::production_module::ProductionModule;
    use crate::construct_module::ConstructModuleType::Production;
    use crate::products::Product;
    use crate::sector::SectorPosition;

    #[test]
    fn load_and_unload_tries_its_best() {
        let sector_position = ConstructPositionSector::new(SectorPosition::new(1, 1, 1), 0);
        let mut construct = Construct::new("The base".to_string(), 500, sector_position);
        assert_eq!(None, construct.current_storage.get(&Product::PowerCells));

        assert_eq!(500, request_load(&mut construct, Amount::new(Product::PowerCells, 700)));
        assert_eq!(500, *construct.current_storage.get(&Product::PowerCells).unwrap());

        assert_eq!(500, request_unload(&mut construct, Amount::new(Product::PowerCells, 700)));
        assert_eq!(None, construct.current_storage.get(&Product::PowerCells));

        assert_eq!(500, request_load(&mut construct, Amount::new(Product::PowerCells, 700)));
        assert_eq!(500, *construct.current_storage.get(&Product::PowerCells).unwrap());

        assert_eq!(700, request_load(&mut construct, Amount::new(Product::PowerCells, 700)));
        assert_eq!(500, *construct.current_storage.get(&Product::PowerCells).unwrap());

        assert_eq!(500, request_unload(&mut construct, Amount::new(Product::PowerCells, 700)));
        assert_eq!(None, construct.current_storage.get(&Product::PowerCells));

        assert_eq!(0, request_unload(&mut construct, Amount::new(Product::PowerCells, 700)));
        assert_eq!(None, construct.current_storage.get(&Product::PowerCells));
    }

    #[test]
    fn install_and_uninstall_tries_its_best() {
        let sector_position = ConstructPositionSector::new(SectorPosition::new(1, 1, 1), 0);
        let mut construct = Construct::new("The base".to_string(), 500, sector_position);
        let ore_production = ProductionModule::new(
            "PowerToOre".to_string(),
            vec![Amount::new(Product::PowerCells, 1)],
            vec![Amount::new(Product::Ores, 2)],
            1,
            0,
        );
        let metal_production = ProductionModule::new(
            "OreAndEnergyToMetal".to_string(),
            vec![Amount::new(Product::PowerCells, 2), Amount::new(Product::Ores, 4)],
            vec![Amount::new(Product::Metals, 1)],
            3,
            0,
        );

        assert_eq!(Ok(()), construct.install(Production(ore_production.clone())));
        assert_eq!(Ok(()), construct.install(Production(metal_production.clone())));

        assert_eq!(Some(&Production(ore_production.clone())), construct.modules.get(0));
        assert_eq!(Some(&Production(metal_production.clone())), construct.modules.get(1));

        assert_eq!(Err("Module with that name already exists.".to_string()), construct.install(Production(ore_production.clone())));

        assert_eq!(Ok(()), construct.uninstall(&ore_production.name().to_string()));
        assert_eq!(Err("Module with that name is not installed.".to_string()), construct.uninstall(&ore_production.name().to_string()));

        assert_eq!(Some(&Production(metal_production.clone())), construct.modules.get(0));
    }


    #[test]
    fn test_parsing() {
        let sector_position = ConstructPositionSector::new(SectorPosition::new(1, 1, 1), 0);
        let mut construct = Construct::new("The base".to_string(), 500, sector_position);
        format!("{:?}", request_state(&mut construct));
    }


    #[test]
    fn docking() {
        let sector_position = ConstructPositionSector::new(SectorPosition::new(1, 1, 1), 0);
        let mut construct = Construct::new("The base".to_string(), 500, sector_position.clone());
        let construct2 = Construct::new("The base2".to_string(), 500, sector_position.clone());

        assert_eq!(InSector(sector_position.clone()), *construct.position.position());
        assert_eq!(
            ConstructEvenReturnType::ConstructPosition(ConstructPositionEventReturnType::Denied("Construct cannot dock with itself.".to_string())),
            construct.handle_event(&ConstructEventType::External(ExternalConstructEventType::ConstructPosition(ExternalConstructPositionEventType::Dock(construct.name().to_string()))))
        );
        assert_eq!(InSector(sector_position.clone()), *construct.position.position());

        assert_eq!(
            ConstructEvenReturnType::ConstructPosition(ConstructPositionEventReturnType::RequestProcessed),
            construct.handle_event(&ConstructEventType::External(ExternalConstructEventType::ConstructPosition(ExternalConstructPositionEventType::Dock(construct2.name().to_string()))))
        );
        assert_eq!(IsDocked(construct2.name().to_string()), *construct.position.position());

        assert_eq!(
            ConstructEvenReturnType::ConstructPosition(ConstructPositionEventReturnType::Denied("External Undock should never hit construct, use internal dock instead that contains all relevant information".to_string())),
            construct.handle_event(&ConstructEventType::External(ExternalConstructEventType::ConstructPosition(ExternalConstructPositionEventType::Undock)))
        );
        assert_eq!(IsDocked(construct2.name().to_string()), *construct.position.position());

        assert_eq!(
            ConstructEvenReturnType::ConstructPosition(ConstructPositionEventReturnType::RequestProcessed),
            construct.handle_event(&ConstructEventType::Internal(InternalConstructEventType::ConstructPosition(InternalConstructPositionEventType::Undock(sector_position.clone()))))
        );
        assert_eq!(InSector(sector_position.clone()), *construct.position.position());
    }

    #[test]
    fn production() {
        let sector_position = ConstructPositionSector::new(SectorPosition::new(1, 1, 1), 0);
        let mut construct = Construct::new("The base".to_string(), 500, sector_position.clone());
        let ore_production = ProductionModule::new(
            "PowerToOre".to_string(),
            vec![Amount::new(Product::PowerCells, 1)],
            vec![Amount::new(Product::Ores, 2)],
            1,
            0,
        );
        let metal_production = ProductionModule::new(
            "OreAndEnergyToMetal".to_string(),
            vec![Amount::new(Product::PowerCells, 2), Amount::new(Product::Ores, 4)],
            vec![Amount::new(Product::Metals, 1)],
            3,
            0,
        );

        assert_eq!(Ok(()), construct.install(Production(ore_production.clone())));
        assert_eq!(Ok(()), construct.install(Production(metal_production.clone())));
        assert_eq!(None, construct.current_storage.get(&Product::PowerCells));
        assert_eq!(None, construct.current_storage.get(&Product::Ores));
        assert_eq!(None, construct.current_storage.get(&Product::Metals));

        assert_eq!(200, request_load(&mut construct, Amount::new(Product::PowerCells, 200)));

        next_turn(&mut construct, 1);

        assert_eq!(Some(&199), construct.current_storage.get(&Product::PowerCells));
        assert_eq!(None, construct.current_storage.get(&Product::Ores));
        assert_eq!(None, construct.current_storage.get(&Product::Metals));

        next_turn(&mut construct, 2);

        assert_eq!(Some(&198), construct.current_storage.get(&Product::PowerCells));
        assert_eq!(Some(&2), construct.current_storage.get(&Product::Ores));
        assert_eq!(None, construct.current_storage.get(&Product::Metals));

        next_turn(&mut construct, 3);

        assert_eq!(Some(&195), construct.current_storage.get(&Product::PowerCells));
        assert_eq!(None, construct.current_storage.get(&Product::Ores));
        assert_eq!(None, construct.current_storage.get(&Product::Metals));

        next_turn(&mut construct, 4);

        assert_eq!(Some(&194), construct.current_storage.get(&Product::PowerCells));
        assert_eq!(Some(&2), construct.current_storage.get(&Product::Ores));
        assert_eq!(None, construct.current_storage.get(&Product::Metals));

        next_turn(&mut construct, 5);

        assert_eq!(Some(&193), construct.current_storage.get(&Product::PowerCells));
        assert_eq!(Some(&4), construct.current_storage.get(&Product::Ores));
        assert_eq!(None, construct.current_storage.get(&Product::Metals));

        next_turn(&mut construct, 6);

        assert_eq!(Some(&190), construct.current_storage.get(&Product::PowerCells));
        assert_eq!(Some(&2), construct.current_storage.get(&Product::Ores));
        assert_eq!(Some(&1), construct.current_storage.get(&Product::Metals));

        for i in 7..200 {
            next_turn(&mut construct, i);
        }

        assert_eq!(None, construct.current_storage.get(&Product::PowerCells));
        assert_eq!(Some(&80), construct.current_storage.get(&Product::Ores));
        assert_eq!(Some(&40), construct.current_storage.get(&Product::Metals)); //(200-80/2)/4

        for i in 201..205 {
            next_turn(&mut construct, i);
        }

        assert_eq!(None, construct.current_storage.get(&Product::PowerCells));
        assert_eq!(Some(&80), construct.current_storage.get(&Product::Ores));
        assert_eq!(Some(&40), construct.current_storage.get(&Product::Metals));

        //Bigger output than input will fill up the station over time
        let metal_production = ProductionModule::new(
            "MetalToEnergy".to_string(),
            vec![Amount::new(Product::Metals, 1)],
            vec![Amount::new(Product::PowerCells, 200)],
            1,
            0,
        );
        assert_eq!(Ok(()), construct.install(Production(metal_production.clone())));

        next_turn(&mut construct, 206);

        assert_eq!(None, construct.current_storage.get(&Product::PowerCells));
        assert_eq!(Some(&80), construct.current_storage.get(&Product::Ores));
        assert_eq!(Some(&39), construct.current_storage.get(&Product::Metals));

        next_turn(&mut construct, 207);

        assert_eq!(Some(&200), construct.current_storage.get(&Product::PowerCells));
        assert_eq!(Some(&80), construct.current_storage.get(&Product::Ores));
        assert_eq!(Some(&38), construct.current_storage.get(&Product::Metals));

        next_turn(&mut construct, 208);

        assert_eq!(Some(&197), construct.current_storage.get(&Product::PowerCells));
        assert_eq!(Some(&76), construct.current_storage.get(&Product::Ores));
        assert_eq!(Some(&38), construct.current_storage.get(&Product::Metals));

        next_turn(&mut construct, 209);

        assert_eq!(Some(&196), construct.current_storage.get(&Product::PowerCells));
        assert_eq!(Some(&78), construct.current_storage.get(&Product::Ores));
        assert_eq!(Some(&38), construct.current_storage.get(&Product::Metals));
    }

    fn request_load(construct: &mut Construct, amount: Amount) -> u32 {
        if let ConstructEvenReturnType::RequestLoadProcessed(loaded_value) = construct.handle_event(&ConstructEventType::Internal(InternalConstructEventType::RequestLoad(amount))) {
            loaded_value
        } else {
            panic!("request_load failed in test")
        }
    }

    fn request_unload(construct: &mut Construct, amount: Amount) -> u32 {
        if let ConstructEvenReturnType::RequestUnloadProcessed(loaded_value) = construct.handle_event(&ConstructEventType::Internal(InternalConstructEventType::RequestUnload(amount))) {
            loaded_value
        } else {
            panic!("request_load failed in test")
        }
    }

    fn next_turn(construct: &mut Construct, turn: u64) {
        match construct.handle_event(&ConstructEventType::Internal(InternalConstructEventType::ExecuteTurn(turn))) {
            ConstructEvenReturnType::TurnExecuted => {}
            _ => panic!("request_load failed in test")
        }
    }

    fn request_state(construct: &mut Construct) -> Construct {
        if let ConstructEvenReturnType::ConstructState(construct) = construct.handle_event(&ConstructEventType::External(ExternalConstructEventType::GetConstructState { include_stack: true })) {
            construct
        } else {
            panic!("request_load failed in test")
        }
    }
}