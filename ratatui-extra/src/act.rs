use ratatui::layout::Position;
use url::Url;

// TODO rename to something like Widget Post Handle Event Actions
pub trait Act: Default {
    fn ignore_esc(&mut self);

    fn ignore_left(&mut self);

    fn ignore_right(&mut self);

    fn is_esc_ignored(&self) -> bool;

    fn merge(&mut self, other: Self);

    fn copy_to_clipboard(&mut self, text: String, tip_position: Option<Position>);

    fn open_url(&mut self, url: Url, tip_position: Option<Position>);
}

#[derive(Default)]
pub struct DefaultAct {
    ignore_esc: bool,
    ignore_left: bool,
    ignore_right: bool,
    copy_to_clipboard: Option<(String, Option<Position>)>,
    open_url: Option<(Url, Option<Position>)>,
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

    fn is_esc_ignored(&self) -> bool {
        self.ignore_esc
    }

    fn copy_to_clipboard(&mut self, text: String, tip_position: Option<Position>) {
        self.copy_to_clipboard = Some((text, tip_position));
    }

    fn open_url(&mut self, url: Url, tip_position: Option<Position>) {
        self.open_url = Some((url, tip_position));
    }

    fn merge(&mut self, other: Self) {
        self.ignore_esc |= other.ignore_esc;
        self.ignore_left |= other.ignore_left;
        self.ignore_right |= other.ignore_right;
        self.copy_to_clipboard = other.copy_to_clipboard.or(self.copy_to_clipboard.take());
        self.open_url = other.open_url.or(self.open_url.take());
    }
}
