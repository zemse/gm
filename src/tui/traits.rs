use std::sync::mpsc;

use ratatui::{
    layout::Rect,
    widgets::{Block, Widget},
};

use super::{app::pages::Page, events::Event};

pub trait BorderedWidget {
    fn render_with_block(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        block: Block<'_>,
    ) where
        Self: Sized;
}

impl<T: Widget> BorderedWidget for T {
    fn render_with_block(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        block: Block<'_>,
    ) where
        Self: Sized,
    {
        let inner_area = block.inner(area);
        block.render(area, buf);
        self.render(inner_area, buf);
    }
}

pub trait WidgetHeight {
    fn height_used(&self, area: ratatui::prelude::Rect) -> u16;
}

#[derive(Default)]
pub struct HandleResult {
    pub page_pops: usize,
    pub page_inserts: Vec<Page>,
    pub reload: bool,
}

pub trait Component {
    fn reload(&mut self) {}

    fn text_input_mut(&mut self) -> Option<&mut String> {
        None
    }

    fn handle_event(
        &mut self,
        event: &Event,
        transmitter: &mpsc::Sender<Event>,
    ) -> crate::Result<HandleResult>;

    fn render_component(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) -> Rect
    where
        Self: Sized;

    fn render_component_with_block(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        block: Block<'_>,
    ) -> Rect
    where
        Self: Sized,
    {
        let inner_area = block.inner(area);
        block.render(area, buf);
        self.render_component(inner_area, buf);
        area
    }
}

// macro_rules! impl_Widget_from_Component {
//     ($t:ty) => {
//         impl ratatui::widgets::Widget for &$t {
//             fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
//             where
//                 Self: Sized,
//             {
//                 self.render_component(area, buf);
//             }
//         }
//     };
// }
