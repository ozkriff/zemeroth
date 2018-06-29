use Action;

#[derive(Debug)]
pub struct Fork {
    action: Option<Box<dyn Action>>,
}

impl Fork {
    pub fn new(action: Box<dyn Action>) -> Self {
        Self {
            action: Some(action),
        }
    }
}

impl Action for Fork {
    fn try_fork(&mut self) -> Option<Box<dyn Action>> {
        self.action.take()
    }

    fn is_finished(&self) -> bool {
        self.action.is_none()
    }

    fn end(&mut self) {
        assert!(self.action.is_none());
    }
}
