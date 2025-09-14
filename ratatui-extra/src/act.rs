pub trait Act: Default {
    fn ignore_esc(&mut self);

    fn merge(&mut self, other: Self);
}

#[derive(Default)]
pub struct DefaultAct {
    ignore_esc: bool,
}

impl Act for DefaultAct {
    fn ignore_esc(&mut self) {
        self.ignore_esc = true;
    }

    fn merge(&mut self, other: Self) {
        self.ignore_esc |= other.ignore_esc;
    }
}
