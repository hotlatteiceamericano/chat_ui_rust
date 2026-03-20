use std::collections::HashMap;

use chat_common::{message::Message, user::User};
use color_eyre::eyre::{self, Context, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::app_event::AppEvents;

#[derive(PartialEq)]
enum FocusedPanel {
    ConversationList,
    ChatInput,
}

pub struct App {
    current_user_id: u32,
    exit: bool,
    focused_panel: FocusedPanel,
    conversations: Vec<User>,
    conversation_state: ListState,
    selected_conversation: Option<usize>,
    input_buffer: String,
    messages: HashMap<u32, Vec<Message>>,
    app_rx: UnboundedReceiver<AppEvents>,
    outbound_tx: UnboundedSender<Message>,
}

impl App {
    pub fn new(
        current_user_id: u32,
        app_rx: UnboundedReceiver<AppEvents>,
        outbound_tx: UnboundedSender<Message>,
    ) -> Self {
        let conversations = vec![User::new(1, "Alice"), User::new(2, "Bob")];
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            current_user_id,
            exit: false,
            focused_panel: FocusedPanel::ConversationList,
            conversations,
            conversation_state: list_state,
            selected_conversation: None,
            input_buffer: String::new(),
            messages: HashMap::new(),
            app_rx,
            outbound_tx,
        }
    }

    /// the main loop to draw the UI and listen for events
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            match self.app_rx.recv().await {
                Some(AppEvents::InboundMessage { message }) => {
                    self.handle_incoming_message(message)?;
                }
                Some(AppEvents::KeyEvent { key_event }) => {
                    self.handle_key_event(key_event)?;
                }
                None => {}
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        // Global keybindings
        match (key_event.modifiers, key_event.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('q')) => {
                self.exit = true;
                return Ok(());
            }
            (_, KeyCode::Tab) => {
                if self.selected_conversation.is_some() {
                    self.focused_panel = match self.focused_panel {
                        FocusedPanel::ConversationList => FocusedPanel::ChatInput,
                        FocusedPanel::ChatInput => FocusedPanel::ConversationList,
                    };
                }
                return Ok(());
            }
            (_, KeyCode::Esc) => {
                self.focused_panel = FocusedPanel::ConversationList;
                return Ok(());
            }
            _ => {}
        }

        // Panel-specific keybindings
        match self.focused_panel {
            FocusedPanel::ConversationList => match key_event.code {
                KeyCode::Up | KeyCode::Char('k') => self.move_up(),
                KeyCode::Down | KeyCode::Char('j') => self.move_down(),
                KeyCode::Enter => {
                    self.selected_conversation = self.conversation_state.selected();
                    self.focused_panel = FocusedPanel::ChatInput;
                    self.input_buffer.clear();
                }
                _ => {}
            },
            FocusedPanel::ChatInput => match key_event.code {
                KeyCode::Char(c) => self.input_buffer.push(c),
                KeyCode::Backspace => {
                    self.input_buffer.pop();
                }
                KeyCode::Enter => {
                    self.send_msg()?;
                    self.input_buffer.clear();
                }
                _ => {}
            },
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
        if self.conversations.is_empty() {
            return;
        }
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
        // todo: move the conversation list to its own struct
        let sidebar_border_style = if self.focused_panel == FocusedPanel::ConversationList {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };
        let conversation_list = Block::default()
            .title(" Conversations ")
            .borders(Borders::ALL)
            .border_style(sidebar_border_style);

        let items: Vec<ListItem> = self
            .conversations
            .iter()
            .map(|u| ListItem::from(u))
            .collect();

        let list = List::new(items).block(conversation_list).highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

        frame.render_stateful_widget(list, layout[0], &mut self.conversation_state);

        // --- Right panel ---
        // todo: move the chat window to its own struct
        // conditionally render when there selected_conversation is Some or None
        match self.selected_conversation {
            Some(idx) => {
                let user = &self.conversations[idx];

                let right_layout = Layout::vertical([
                    Constraint::Length(3), // header
                    Constraint::Min(1),    // messages
                    Constraint::Length(3), // input
                ])
                .split(layout[1]);

                // Header
                let header = Paragraph::new(Line::from(user.display_name()))
                    .block(Block::default().borders(Borders::ALL));
                frame.render_widget(header, right_layout[0]);

                // Message area
                let msg_block = Block::default().borders(Borders::ALL);
                let inner_width = right_layout[1].width.saturating_sub(2) as usize; // subtract border
                let mut lines: Vec<Line> = Vec::new();

                if let Some(chat_messages) = self.messages.get(&user.id()) {
                    for msg in chat_messages {
                        let is_mine = msg.sender_id == self.current_user_id;
                        let indicator = if is_mine { "▓ " } else { "░ " };
                        let indicator_style = if is_mine {
                            Style::default().fg(Color::White)
                        } else {
                            Style::default().fg(Color::White)
                        };
                        let text = format!("{}{}", indicator, msg.payload);
                        let text_width = text.len();

                        if is_mine {
                            let padding = inner_width.saturating_sub(text_width);
                            let padded = format!("{:>width$}", text, width = padding + text_width);
                            lines.push(Line::from(Span::styled(padded, indicator_style)));
                        } else {
                            lines.push(Line::from(Span::styled(text, indicator_style)));
                        }
                    }
                }

                let visible_height = right_layout[1].height.saturating_sub(2) as usize;
                let scroll_offset = lines.len().saturating_sub(visible_height) as u16;
                let messages_widget = Paragraph::new(lines)
                    .block(msg_block)
                    .scroll((scroll_offset, 0));
                frame.render_widget(messages_widget, right_layout[1]);

                // Input box
                let input_border_style = if self.focused_panel == FocusedPanel::ChatInput {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default()
                };
                let input_text = if self.input_buffer.is_empty() {
                    Line::from(Span::styled(
                        "Type a message...",
                        Style::default().fg(Color::DarkGray),
                    ))
                } else {
                    Line::from(self.input_buffer.as_str())
                };
                let input = Paragraph::new(input_text).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(input_border_style),
                );
                frame.render_widget(input, right_layout[2]);
            }
            None => {
                let empty = Block::default().borders(Borders::ALL);
                frame.render_widget(empty, layout[1]);
            }
        }
    }

    fn send_msg(&mut self) -> Result<()> {
        if self.input_buffer.is_empty() {
            return Ok(());
        }
        let user = self
            .conversations
            .get(self.selected_conversation.ok_or(eyre::eyre!(
                "no conversation seleceted when sending messages"
            ))?)
            .ok_or(eyre::eyre!("conversation not found when sending messages"))?;
        let receiver_id = user.id();
        let msg = Message {
            sender_id: self.current_user_id,
            receiver_id,
            payload: self.input_buffer.to_string(),
        };

        self.outbound_tx
            .send(msg)
            .context("failed sending message to sending receiver")?;

        let stored_msg = Message {
            sender_id: self.current_user_id,
            receiver_id,
            payload: self.input_buffer.to_string(),
        };
        self.messages
            .entry(receiver_id)
            .or_default()
            .push(stored_msg);

        Ok(())
    }

    fn handle_incoming_message(&mut self, message: Message) -> Result<()> {
        let sender_id = message.sender_id;
        self.messages.entry(sender_id).or_default().push(message);
        Ok(())
    }
}
