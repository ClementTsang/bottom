#[derive(Debug)]
pub enum ScrollDirection {
    // UP means scrolling up --- this usually DECREMENTS
    Up,
    // DOWN means scrolling down --- this usually INCREMENTS
    Down,
}

impl Default for ScrollDirection {
    fn default() -> Self {
        ScrollDirection::Down
    }
}
