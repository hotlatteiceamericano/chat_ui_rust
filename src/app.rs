use std::{collections::HashMap, sync::Arc};

use chat_common::{message::Message, user::User};
use color_eyre::eyre::{self, Context, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{app_event::AppEvents, http_server::HttpServer};

#[derive(PartialEq)]
enum FocusedPanel {
    ConversationList,
    ChatInput,
}

enum Modal {
    None,
    NewConversation {
        input: String,
        error: Option<String>,
        pending: bool,
    },
}

pub struct App {
    current_user_id: String,
    exit: bool,
    focused_panel: FocusedPanel,
    modal: Modal,
    conversations: Vec<User>,
    conversation_state: ListState,
    selected_conversation: Option<usize>,
    input_buffer: String,
    messages: HashMap<String, Vec<Message>>,
    app_rx: UnboundedReceiver<AppEvents>,
    app_tx: UnboundedSender<AppEvents>,
    outbound_tx: UnboundedSender<Message>,
    http_server: Arc<HttpServer>,
}

impl App {
    pub fn new(
        current_user_id: String,
        app_rx: UnboundedReceiver<AppEvents>,
        app_tx: UnboundedSender<AppEvents>,
        outbound_tx: UnboundedSender<Message>,
        http_server: Arc<HttpServer>,
    ) -> Self {
        // todo: read from the conversation table
        let conversations = vec![
            // User::new("Alice", String::from("alice@chat.com")),
            // User::new("Bob", String::from("Bob@chat.com")),
        ];
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            current_user_id,
            exit: false,
            focused_panel: FocusedPanel::ConversationList,
            modal: Modal::None,
            conversations,
            conversation_state: list_state,
            selected_conversation: None,
            input_buffer: String::new(),
            messages: HashMap::new(),
            app_rx,
            app_tx,
            outbound_tx,
            http_server,
        }
    }

    /// the main loop to draw the UI and listen for events
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        while !self.exit {
            let mut draw_result: Result<()> = Ok(());
            terminal.draw(|frame| {
                draw_result = self.draw(frame);
            })?;
            draw_result.context("failed to draw frame")?;
            match self.app_rx.recv().await {
                Some(AppEvents::InboundMessage { message }) => {
                    self.handle_incoming_message(message)?;
                }
                Some(AppEvents::KeyEvent { key_event }) => {
                    self.handle_key_event(key_event)?;
                }
                Some(AppEvents::UserLookupResult(result)) => {
                    self.handle_user_lookup_result(result);
                }
                None => {}
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        // Modal owns all keys while open (except Ctrl-Q to quit).
        if matches!(self.modal, Modal::NewConversation { .. }) {
            if key_event.modifiers == KeyModifiers::CONTROL && key_event.code == KeyCode::Char('q')
            {
                self.exit = true;
                return Ok(());
            }
            self.handle_modal_key_event(key_event)?;
            return Ok(());
        }

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
                KeyCode::Char('n') => {
                    self.modal = Modal::NewConversation {
                        input: String::new(),
                        error: None,
                        pending: false,
                    };
                }
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

    fn handle_modal_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        let Modal::NewConversation {
            input,
            error,
            pending,
        } = &mut self.modal
        else {
            return Ok(());
        };

        if *pending {
            // Ignore input while a lookup is in flight, except Esc to dismiss.
            if key_event.code == KeyCode::Esc {
                self.modal = Modal::None;
            }
            return Ok(());
        }

        match key_event.code {
            KeyCode::Esc => {
                self.modal = Modal::None;
            }
            KeyCode::Backspace => {
                input.pop();
                *error = None;
            }
            KeyCode::Char(c) => {
                input.push(c);
                *error = None;
            }
            KeyCode::Enter => {
                let email = input.trim().to_string();
                if email.is_empty() {
                    *error = Some("email cannot be empty".to_string());
                    return Ok(());
                }

                // Dedupe: if we already have a conversation with this email,
                // refocus it instead of making the HTTP request.
                if let Some(idx) = self
                    .conversations
                    .iter()
                    .position(|u| u.email().eq_ignore_ascii_case(&email))
                {
                    self.focus_conversation(idx);
                    self.modal = Modal::None;
                    return Ok(());
                }

                *pending = true;
                let http = self.http_server.clone();
                let tx = self.app_tx.clone();
                tokio::spawn(async move {
                    let result = http
                        .get_user_by_email(&email)
                        .await
                        .map_err(|e| e.to_string());
                    let _ = tx.send(AppEvents::UserLookupResult(result));
                });
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_user_lookup_result(&mut self, result: Result<User, String>) {
        if !matches!(self.modal, Modal::NewConversation { .. }) {
            return;
        }
        match result {
            Ok(user) => {
                // Race-safe dedupe in case the user was added meanwhile.
                let idx = match self
                    .conversations
                    .iter()
                    .position(|u| u.email().eq_ignore_ascii_case(user.email()))
                {
                    Some(idx) => idx,
                    None => {
                        self.conversations.push(user);
                        self.conversations.len() - 1
                    }
                };
                self.focus_conversation(idx);
                self.modal = Modal::None;
            }
            Err(e) => {
                if let Modal::NewConversation { error, pending, .. } = &mut self.modal {
                    *pending = false;
                    *error = Some(e);
                }
            }
        }
    }

    fn focus_conversation(&mut self, idx: usize) {
        self.conversation_state.select(Some(idx));
        self.selected_conversation = Some(idx);
        self.focused_panel = FocusedPanel::ChatInput;
        self.input_buffer.clear();
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

    fn draw(&mut self, frame: &mut Frame) -> Result<()> {
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
            .map(|u| ListItem::new(Line::from(u.email().to_string())))
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

        if let Modal::NewConversation {
            input,
            error,
            pending,
        } = &self.modal
        {
            let modal_area = centered_rect(60, 7, area);
            frame.render_widget(Clear, modal_area);

            let block = Block::default()
                .title(" New conversation ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));
            let inner = block.inner(modal_area);
            frame.render_widget(block, modal_area);

            let rows = Layout::vertical([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(inner);

            let prompt = Paragraph::new(Line::from("Recipient email:"));
            frame.render_widget(prompt, rows[0]);

            let input_line: Line = if *pending {
                Line::from(Span::styled(
                    format!("{}  (looking up...)", input),
                    Style::default().fg(Color::DarkGray),
                ))
            } else if input.is_empty() {
                Line::from(Span::styled(
                    "type an email and press Enter",
                    Style::default().fg(Color::DarkGray),
                ))
            } else {
                Line::from(input.as_str())
            };
            frame.render_widget(Paragraph::new(input_line), rows[1]);

            let status_line: Line = match error {
                Some(e) => Line::from(Span::styled(e.as_str(), Style::default().fg(Color::Red))),
                None => Line::from(Span::styled(
                    "Esc to cancel",
                    Style::default().fg(Color::DarkGray),
                )),
            };
            frame.render_widget(Paragraph::new(status_line), rows[2]);
        }

        Ok(())
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
            sender_id: self.current_user_id.clone(),
            receiver_id: receiver_id.clone(),
            payload: self.input_buffer.to_string(),
        };

        self.outbound_tx
            .send(msg)
            .context("failed sending message to sending receiver")?;

        let stored_msg = Message {
            sender_id: self.current_user_id.clone(),
            receiver_id: receiver_id.clone(),
            payload: self.input_buffer.to_string(),
        };
        self.messages
            .entry(receiver_id.clone())
            .or_default()
            .push(stored_msg);

        Ok(())
    }

    fn handle_incoming_message(&mut self, message: Message) -> Result<()> {
        let sender_id = &message.sender_id;
        self.messages
            .entry(sender_id.to_owned())
            .or_default()
            .push(message);
        Ok(())
    }
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect {
        x,
        y,
        width: width.min(area.width),
        height: height.min(area.height),
    }
}
