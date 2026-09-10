use crate::app::App;
use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, List, ListItem, ListState},
    Frame,
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let tree_block = Block::default();

    let items: Vec<ListItem> = app
        .items
        .iter()
        .map(|item| {
            let indent = "  ".repeat(item.depth);
            let icon = if item.is_parent_link {
                "󰕒"
            } else if item.is_dir {
                if item.is_expanded {
                    ""
                } else {
                    ""
                }
            } else {
                ""
            };
            let text = format!("{}{} {}", indent, icon, item.name());
            
            // Simple styling
            let style = if item.is_parent_link {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else if item.is_dir {
                Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(text).style(style)
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.selected_index));

    let list = List::new(items)
        .block(tree_block)
        .highlight_style(Style::default().bg(Color::Rgb(60, 60, 60)).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, f.area(), &mut state);
}
