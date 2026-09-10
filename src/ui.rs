use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::App;

pub fn draw(f: &mut Frame, _app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(f.area());

    let tree_block = Block::default().title("Tree").borders(Borders::ALL);
    let tree_para = Paragraph::new("Directory Tree Placeholder").block(tree_block);
    f.render_widget(tree_para, chunks[0]);

    let preview_block = Block::default().title("Preview").borders(Borders::ALL);
    let preview_para = Paragraph::new("Preview Area Placeholder").block(preview_block);
    f.render_widget(preview_para, chunks[1]);
}
