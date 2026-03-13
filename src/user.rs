use ratatui::{text::Line, widgets::ListItem};

pub struct User {
    id: u32,
    display_name: String,
}

impl User {
    pub fn new(id: u32, display_name: &str) -> Self {
        Self {
            id,
            display_name: display_name.to_string(),
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn display_name(&self) -> &str {
        self.display_name.as_str()
    }
}

impl<'a> From<&'a User> for ListItem<'a> {
    fn from(value: &'a User) -> ListItem<'a> {
        ListItem::new(Line::from(format!(
            "{}({})",
            value.display_name(),
            value.id()
        )))
    }
}
