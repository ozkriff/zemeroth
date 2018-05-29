use Action;

#[derive(Debug)]
pub struct Empty;

impl Empty {
    pub fn new() -> Self {
        Empty
    }
}

impl Action for Empty {}
