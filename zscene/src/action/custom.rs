use std::fmt;

use crate::Action;

pub struct Custom {
    f: Box<dyn FnMut()>,
}

impl fmt::Debug for Custom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Custom").field("f", &"").finish()
    }
}

impl Custom {
    pub fn new(f: Box<dyn FnMut()>) -> Self {
        Self { f }
    }
}

impl Action for Custom {
    fn begin(&mut self) {
        (self.f)();
    }
}
