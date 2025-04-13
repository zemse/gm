use strum::IntoEnumIterator;

use crate::actions::{setup::get_setup_menu, Action};

pub struct Navigation {
    on_left: bool,
    left_pane: Vec<Level>,
    right_pane: Vec<Level>,
}

struct Level {
    cursor: usize,
    max_items: usize,
}

impl Default for Navigation {
    fn default() -> Self {
        Self {
            on_left: true,
            left_pane: vec![Level {
                cursor: 0,
                max_items: Action::iter().count(),
            }],
            right_pane: vec![Level {
                cursor: 0,
                max_items: 1,
            }],
        }
    }
}

impl Navigation {
    pub fn left_list(&self) -> Vec<String> {
        // TODO implement this properly

        match self.left_pane.len() {
            _ => Action::iter()
                .map(|action| format!("{action}"))
                .collect::<Vec<String>>(),
            // 2 => match &Action::iter().collect::<Vec<Action>>()[self.left_pane[0].cursor] {
            //     Action::Setup => get_setup_menu()
            //         .into_iter()
            //         .map(|item| item.to_string())
            //         .collect(),
            //     Action::Assets => todo!(),
            //     Action::Account { .. } => todo!(),
            //     Action::AddressBook { .. } => todo!(),
            //     Action::Transaction { .. } => todo!(),
            //     Action::SignMessage { .. } => todo!(),
            //     Action::SendMessage { .. } => todo!(),
            //     Action::Config { .. } => todo!(),
            // },
            // _ => vec![],
        }
    }

    pub fn left_idx(&self) -> Option<usize> {
        if self.on_left {
            let level_idx = self.left_pane.len() - 1;
            Some(self.left_pane[level_idx].cursor)
        } else {
            None
        }
    }

    fn level(&mut self) -> &mut Level {
        if self.on_left {
            let level_idx = self.left_pane.len() - 1;
            &mut self.left_pane[level_idx]
        } else {
            let level_idx = self.right_pane.len() - 1;
            &mut self.right_pane[level_idx]
        }
    }

    pub fn up(&mut self) {
        let level = self.level();
        level.cursor = (level.max_items + level.cursor - 1) % level.max_items;
    }

    pub fn down(&mut self) {
        let level = self.level();
        level.cursor = (level.cursor + 1) % level.max_items;
    }

    pub fn right(&mut self) {
        self.on_left = false;
    }

    pub fn left(&mut self) {
        self.on_left = true;
    }

    pub fn enter(&mut self) {
        if self.on_left {
            let new_idx = self.left_pane.len();
            self.left_pane.push(Level {
                cursor: 0,
                max_items: 0,
            });
            self.left_pane[new_idx].max_items = self.left_list().len();
            self.left_list().len();
        } else {
            unimplemented!()
        }
    }

    pub fn esc(&mut self) -> bool {
        if self.on_left {
            if self.left_pane.len() > 1 {
                self.left_pane.pop();
                true
            } else {
                false
            }
        } else {
            unimplemented!()
        }
    }
}
