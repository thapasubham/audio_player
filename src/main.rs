
mod player;
mod terminal;
fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(terminal::app::app)?;
    Ok(())
}
