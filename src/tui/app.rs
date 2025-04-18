use std::io;

use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use pages::{main_menu::MainMenuPage, Page};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, BorderType, Widget},
    DefaultTerminal,
};
use widgets::{footer::Footer, right::Right, title::Title};

use super::{
    events::Event,
    traits::{BorderedWidget, Component},
};

pub mod pages;
pub mod widgets;

pub struct App {
    pub exit: bool,
    pub context: Vec<Page>,
    pub eth_price: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            exit: false,
            context: vec![Page::MainMenu(MainMenuPage::default())],
            eth_price: None,
        }
    }
}

impl App {
    pub fn draw(&self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        terminal.draw(|frame| {
            frame.render_widget(self, frame.area());
        })?;
        Ok(())
    }

    pub fn handle_event(&mut self, event: super::events::Event) {
        if let Some(page) = self.current_page_mut() {
            let result = page.handle_event(&event);
            for _ in 0..result.page_pops {
                self.context.pop();
            }
            if result.reload {
                if let Some(page) = self.current_page_mut() {
                    page.reload();
                }
            }
            self.context.extend(result.page_inserts);
        }

        if self.context.is_empty() {
            self.exit = true;
        }

        match event {
            Event::Input(key_event) => {
                // check if we should exit on 'q' press
                if key_event.kind == KeyEventKind::Press {
                    #[allow(clippy::single_match)]
                    match key_event.code {
                        KeyCode::Char(char) => {
                            // if char == 'q' && self.navigation.text_input().is_none() {
                            //     self.exit = true;
                            // }

                            if char == 'c' && key_event.modifiers == KeyModifiers::CONTROL {
                                self.exit = true;
                            }
                        }
                        KeyCode::Esc => {
                            self.context.pop();
                            if self.context.is_empty() {
                                self.exit = true;
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::EthPriceUpdate(eth_price) => {
                self.eth_price = Some(eth_price);
            }
        };
    }

    fn current_page(&self) -> Option<&Page> {
        self.context.last()
    }

    fn current_page_mut(&mut self) -> Option<&mut Page> {
        self.context.last_mut()
    }
}

// impl_Widget_from_Component!(App);

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let vertical_layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
        ]);
        let [title_area, body_area, footer_area] = vertical_layout.areas(area);

        if let Some(page) = self.current_page() {
            Title.render(title_area, buf);

            // Body render
            if page.is_full_screen() {
                page.render_component_with_block(body_area, buf, Block::bordered());
            } else {
                let horizontal_layout =
                    Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)]);
                let [left_area, right_area] = horizontal_layout.areas(body_area);

                page.render_component_with_block(left_area, buf, Block::bordered());

                Right {
                    eth_price: &self.eth_price,
                }
                .render_with_block(
                    right_area,
                    buf,
                    Block::bordered().border_type(BorderType::Plain),
                );
            }

            Footer {
                exit: &self.exit,
                is_main_menu: &page.is_main_menu(),
            }
            .render(footer_area, buf);
        }
    }
}
