mod time {
    pub struct TimeStack {
        event_stack: Vec<TimeEvent>,
    }

    #[derive(Clone, PartialEq)]
    pub struct TimeEvent {
        time_event_type: TimeEventType,
        index: usize,
    }

    impl TimeEvent {
        pub fn time_event_type(&self) -> &TimeEventType {
            &self.time_event_type
        }
        pub fn index(&self) -> usize {
            self.index
        }

        pub fn new(time_event_type: TimeEventType, index: usize) -> Self {
            TimeEvent { time_event_type, index }
        }
    }

    #[derive(Clone, PartialEq, Debug)]
    pub enum TimeEventType {
        Pause,
        Start,
    }

    impl TimeStack {
        fn add_event(&mut self, time_event_type: TimeEventType) {
            self.event_stack.push(TimeEvent::new(time_event_type, self.event_stack.len()));
        }

        pub fn pause(&mut self) {
            self.add_event(TimeEventType::Pause);
        }

        pub fn start(&mut self) {
            self.add_event(TimeEventType::Start);
        }

        pub fn event_stack_clone(&self) -> Vec<TimeEvent> {
            self.event_stack.clone()
        }

        pub fn new() -> Self {
            TimeStack { event_stack: Vec::new() }
        }
    }
}

#[cfg(test)]
mod tests_int {
    use crate::time::time::{TimeEventType, TimeStack};

    #[test]
    fn it_works() {
        let mut time_stack = TimeStack::new();
        time_stack.pause();
        assert_eq!(1, time_stack.event_stack_clone().len());
        let x1 = time_stack.event_stack_clone();
        assert_eq!(&TimeEventType::Pause, x1.get(0).unwrap().time_event_type());
        assert_eq!(0, x1.get(0).unwrap().index());
        time_stack.start();
        assert_eq!(2, time_stack.event_stack_clone().len());
        let x2 = time_stack.event_stack_clone();
        assert_eq!(&TimeEventType::Start, x2.get(1).unwrap().time_event_type());
        assert_eq!(1, x2.get(1).unwrap().index());
    }
}
