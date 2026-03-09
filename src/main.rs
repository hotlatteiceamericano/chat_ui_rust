use anyhow::Result;

use crate::app::App;

pub mod app;

fn main() -> Result<()> {
    ratatui::run(|terminal| App::new().run(terminal))
}
