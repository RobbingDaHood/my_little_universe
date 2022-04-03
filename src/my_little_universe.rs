use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{ExternalCommandReturnValues, ExternalCommands};
use crate::construct::construct::{Construct, ConstructEventType, InternalConstructEventType};
use crate::save_load::ExternalSaveLoad;
use crate::time::{InternalTimeEventType, TimeEventType, TimeStackState};

pub struct MyLittleUniverse {
    time: TimeStackState,
    pub(crate) constructs: HashMap<String, Construct>,
    universe_name: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum MyLittleUniverseReturnValues {
    CouldNotFindStation,
    CouldNotFindConstruct(String),
}

impl MyLittleUniverse {
    pub fn new(universe_name: String, time: TimeStackState, constructs: HashMap<String, Construct>) -> Self {
        MyLittleUniverse {
            time,
            universe_name,
            constructs,
        }
    }

    pub fn time(&self) -> &TimeStackState {
        &self.time
    }

    pub fn constructs(&self) -> &HashMap<String, Construct> {
        &self.constructs
    }
    pub fn universe_name(&self) -> &str {
        &self.universe_name
    }

    pub fn handle_event(&mut self, event: ExternalCommands) -> ExternalCommandReturnValues {
        match event {
            ExternalCommands::Time(time_event) => {
                let return_type = self.time.push_event(&TimeEventType::External(time_event));
                ExternalCommandReturnValues::Time(return_type)
            }
            ExternalCommands::Construct(construct_name, construct_event) => {
                return match self.constructs.get_mut(&construct_name) {
                    Some(construct) => {
                        let return_type = construct.push_event(&ConstructEventType::External(construct_event));
                        ExternalCommandReturnValues::Construct(return_type)
                    }
                    None => { ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::CouldNotFindConstruct(construct_name)) }
                };
            }
            ExternalCommands::Save(save_event) => {
                match save_event {
                    ExternalSaveLoad::TheUniverseAs(universe_name) => {
                        ExternalCommandReturnValues::Save(self.save_as(&universe_name))
                    }
                    ExternalSaveLoad::TheUniverse => {
                        ExternalCommandReturnValues::Save(self.save())
                    }
                }
            }
        }
    }

    pub fn request_execute_turn(&mut self) {
        if self.time.request_execute_turn() {
            for construct in self.constructs.values_mut() {
                construct.push_event(&ConstructEventType::Internal(InternalConstructEventType::ExecuteTurn(self.time.turn())));
            }
            self.time.push_event(&TimeEventType::Internal(InternalTimeEventType::ReadyForNextTurn));
        }
    }
}


#[cfg(test)]
mod tests_int {
    use std::collections::HashMap;

    use crate::{ExternalCommandReturnValues, ExternalCommands};
    use crate::construct::amount::Amount;
    use crate::construct::construct::{Construct, ConstructEvenReturnType, ExternalConstructEventType};
    use crate::construct::production_module::ProductionModule;
    use crate::construct_module::ConstructModuleType::Production;
    use crate::my_little_universe::{MyLittleUniverse, MyLittleUniverseReturnValues};
    use crate::products::Product;
    use crate::time::{ExternalTimeEventType, TimeEventReturnType, TimeStackState};

    #[test]
    fn it_works() {
        //Setup universe
        let mut construct = Construct::new("The base".to_string(), 500);
        let ore_production = ProductionModule::new(
            "PowerToOre".to_string(),
            vec![Amount::new(Product::PowerCells, 1)],
            vec![Amount::new(Product::Ores, 2)],
            1,
            0,
        );
        assert_eq!(Ok(()), construct.install(Production(ore_production.clone())));

        let mut constructs: HashMap<String, Construct> = HashMap::new();
        constructs.insert(construct.name().to_string(), construct);

        let mut universe = MyLittleUniverse::new("universe_name".to_string(), TimeStackState::new(), constructs);

        //testing
        assert_eq!(
            ExternalCommandReturnValues::Construct(ConstructEvenReturnType::RequestLoadProcessed(0)),
            universe.handle_event(ExternalCommands::Construct("The base".to_string(), ExternalConstructEventType::RequestLoad(Amount::new(Product::PowerCells, 200))))
        );
        assert_eq!(
            ExternalCommandReturnValues::Construct(ConstructEvenReturnType::RequestUnloadProcessed(0)),
            universe.handle_event(ExternalCommands::Construct("The base".to_string(), ExternalConstructEventType::RequestUnload(Amount::new(Product::Ores, 2))))
        );

        assert_eq!(
            ExternalCommandReturnValues::Time(TimeEventReturnType::Received),
            universe.handle_event(ExternalCommands::Time(ExternalTimeEventType::StartUntilTurn(100)))
        );
        universe.request_execute_turn();
        universe.request_execute_turn();

        assert!(
            matches!(
                universe.handle_event(ExternalCommands::Construct("The base".to_string(), ExternalConstructEventType::GetConstructState{include_stack: false})),
                ExternalCommandReturnValues::Construct(ConstructEvenReturnType::ConstructState{..})
            )
        );

        assert_eq!(
            ExternalCommandReturnValues::Construct(ConstructEvenReturnType::RequestUnloadProcessed(2)),
            universe.handle_event(ExternalCommands::Construct("The base".to_string(), ExternalConstructEventType::RequestUnload(Amount::new(Product::Ores, 2))))
        );

        assert_eq!(
            ExternalCommandReturnValues::Universe(MyLittleUniverseReturnValues::CouldNotFindConstruct("!The base".to_string())),
            universe.handle_event(ExternalCommands::Construct("!The base".to_string(), ExternalConstructEventType::GetConstructState { include_stack: false })),
        );
    }
}