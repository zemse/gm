// TODO rename to something like Widget Post Handle Event Actions
pub trait Act: Default {
    fn ignore_esc(&mut self);

    fn ignore_left(&mut self);

    fn ignore_right(&mut self);

    fn merge(&mut self, other: Self);
}

#[derive(Default)]
pub struct DefaultAct {
    ignore_esc: bool,
    ignore_left: bool,
    ignore_right: bool,
}

impl Act for DefaultAct {
    fn ignore_esc(&mut self) {
        self.ignore_esc = true;
    }

    fn ignore_left(&mut self) {
        self.ignore_left = true;
    }

    fn ignore_right(&mut self) {
        self.ignore_right = true;
    }

    fn merge(&mut self, other: Self) {
        self.ignore_esc |= other.ignore_esc;
        self.ignore_left |= other.ignore_left;
        self.ignore_right |= other.ignore_right;
    }
}
