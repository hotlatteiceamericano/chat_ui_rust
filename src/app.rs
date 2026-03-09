use anyhow::Result;
use ratatui::{
    DefaultTerminal, Frame, border,
    buffer::Buffer,
    layout::Rect,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

pub struct App {}

impl App {
    pub fn new() -> Self {
        Self {}
    }

    pub fn exit(&self) -> bool {
        false
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        // connect to websocket
        while !self.exit() {
            terminal.draw(|frame| self.draw(frame))?;
        }

        // disconnect from websocket
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(Line::from(" Counter App Tutorial ").centered())
            .border_set(border::THICK);
        Paragraph::new(Text::from("hello"))
            .centered()
            .block(block)
            .render(area, buf);
    }
}
