use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

pub struct App {
    exit: bool,
    conversations: Vec<String>,
    conversation_state: ListState,
    selected_conversation: Option<usize>,
}

impl App {
    pub fn new() -> Self {
        let conversations = vec!["Alice".to_string(), "Bob".to_string()];
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            exit: false,
            conversations,
            conversation_state: list_state,
            selected_conversation: None,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        if let Event::Key(key) = event::read()? {
            match (key.modifiers, key.code) {
                (KeyModifiers::CONTROL, KeyCode::Char('q')) => self.exit = true,
                (_, KeyCode::Up) => self.move_up(),
                (_, KeyCode::Char('k')) => self.move_up(),
                (_, KeyCode::Down) => self.move_down(),
                (_, KeyCode::Char('j')) => self.move_down(),
                (_, KeyCode::Enter) => {
                    self.selected_conversation = self.conversation_state.selected();
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn move_up(&mut self) {
        let i = match self.conversation_state.selected() {
            Some(i) if i > 0 => i - 1,
            Some(i) => i,
            None => 0,
        };
        self.conversation_state.select(Some(i));
    }

    fn move_down(&mut self) {
        let i = match self.conversation_state.selected() {
            Some(i) if i < self.conversations.len() - 1 => i + 1,
            Some(i) => i,
            None => 0,
        };
        self.conversation_state.select(Some(i));
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        // Split into sidebar and chat panel
        let layout = Layout::horizontal([Constraint::Length(40), Constraint::Min(1)]).split(area);

        // --- Conversation List ---
        let conversation_list = Block::default()
            .title(" Conversations ")
            .borders(Borders::ALL);

        let items: Vec<ListItem> = self
            .conversations
            .iter()
            .map(|name| ListItem::new(Line::from(Span::raw(name.as_str()))))
            .collect();

        let list = List::new(items).block(conversation_list).highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

        frame.render_stateful_widget(list, layout[0], &mut self.conversation_state);

        // --- Right panel ---
        match self.selected_conversation {
            Some(idx) => {
                let name = &self.conversations[idx];

                let right_layout = Layout::vertical([
                    Constraint::Length(3), // header
                    Constraint::Min(1),    // messages
                    Constraint::Length(3), // input
                ])
                .split(layout[1]);

                // Header
                let header = Paragraph::new(Line::from(name.as_str()))
                    .block(Block::default().borders(Borders::ALL));
                frame.render_widget(header, right_layout[0]);

                // Message area (empty)
                let messages = Block::default().borders(Borders::ALL);
                frame.render_widget(messages, right_layout[1]);

                // Input box
                let input = Paragraph::new(Line::from("Type a message..."))
                    .style(Style::default().fg(Color::DarkGray))
                    .block(Block::default().borders(Borders::ALL));
                frame.render_widget(input, right_layout[2]);
            }
            None => {
                let empty = Block::default().borders(Borders::ALL);
                frame.render_widget(empty, layout[1]);
            }
        }
    }
}
