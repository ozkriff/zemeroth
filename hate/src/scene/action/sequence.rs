use std::collections::VecDeque;
use std::time::Duration;
use scene::Action;

#[derive(Debug)]
pub struct Sequence {
    actions: VecDeque<Box<Action>>,
    duration: Duration,
}

impl Sequence {
    pub fn new(actions: Vec<Box<Action>>) -> Self {
        let mut total_time = Duration::new(0, 0);
        for action in &actions {
            total_time += action.duration();
        }
        Self {
            actions: actions.into(),
            duration: total_time,
        }
    }

    /// Current action
    fn action(&mut self) -> &mut Action {
        &mut **self.actions.front_mut().unwrap()
    }

    fn end_current_action_and_start_next(&mut self) {
        assert!(!self.actions.is_empty());
        assert!(self.action().is_finished());
        self.action().end();
        self.actions.pop_front().unwrap();
        if !self.actions.is_empty() {
            self.action().begin();
        }
    }
}

impl Action for Sequence {
    fn duration(&self) -> Duration {
        self.duration
    }

    fn begin(&mut self) {
        if !self.actions.is_empty() {
            self.action().begin();
        }
    }

    fn update(&mut self, dtime: Duration) {
        if self.actions.is_empty() {
            return;
        }
        self.action().update(dtime);
        // Skipping instant actions
        while !self.actions.is_empty() && self.action().is_finished() {
            self.end_current_action_and_start_next();
        }
    }

    fn end(&mut self) {
        assert!(self.actions.is_empty());
    }

    fn is_finished(&self) -> bool {
        self.actions.is_empty()
    }

    fn try_fork(&mut self) -> Option<Box<Action>> {
        if self.actions.is_empty() {
            return None;
        }
        let forked_action = self.action().try_fork();
        if forked_action.is_some() && self.action().is_finished() {
            self.end_current_action_and_start_next();
        }
        forked_action
    }
}
