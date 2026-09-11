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

    #[arg(long)]
    toggle_hidden: Option<String>,
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

    if let Some(target) = args.toggle_hidden {
        let sock = format!("/tmp/tmux_nav_{}.sock", target);
        if std::path::Path::new(&sock).exists() {
            let mut cmd = std::process::Command::new("nc");
            cmd.args(["-U", "-N", &sock]);
            use std::io::Write;
            if let Ok(mut child) = cmd.stdin(std::process::Stdio::piped()).spawn() {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(b"!TOGGLE_HIDDEN");
                }
                let _ = child.wait();
            }
        }
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
