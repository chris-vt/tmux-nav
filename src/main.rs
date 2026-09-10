mod app;
mod fs;
mod ipc;
mod ui;

use app::App;
use clap::Parser;
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, stdout};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    target_pane: Option<String>,

    #[arg(long)]
    init_zsh: bool,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    if args.init_zsh {
        println!(r#"
chpwd() {{
    if [[ -n "$TMUX_PANE" ]]; then
        local sock="/tmp/tmux_nav_${{TMUX_PANE}}.sock"
        if [[ -S "$sock" ]]; then
            echo "$PWD" | nc -U -N "$sock" >/dev/null 2>&1 &!
        fi
    fi
}}
"#);
        return Ok(());
    }

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut app = App::new(args.target_pane)?;
    let res = app.run(&mut terminal);

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    res
}
