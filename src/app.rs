use crate::fs::{read_dir, FsItem};
use crate::ui;
use crossterm::event::{self, Event, KeyCode};
use notify::{RecursiveMode, Watcher};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::env;
use std::io;
use std::path::PathBuf;

pub struct App {
    pub should_quit: bool,
    pub root: PathBuf,
    pub items: Vec<FsItem>,
    pub selected_index: usize,
    pub target_pane: Option<String>,
    pub rx: Option<std::sync::mpsc::Receiver<String>>,
    // We hold the watcher to keep it alive
    pub _watcher: Option<notify::RecommendedWatcher>,
}

impl App {
    fn read_root(root: &PathBuf) -> io::Result<Vec<FsItem>> {
        let mut items = vec![];
        if let Some(parent) = root.parent() {
            items.push(FsItem {
                path: parent.to_path_buf(),
                depth: 0,
                is_dir: true,
                is_expanded: false,
                is_parent_link: true,
            });
        }
        if let Ok(mut children) = read_dir(root, 0) {
            items.append(&mut children);
        }
        Ok(items)
    }

    pub fn new(target_pane: Option<String>) -> io::Result<Self> {
        let root = env::current_dir()?;
        let items = Self::read_root(&root)?;

        let (tx, rx) = std::sync::mpsc::channel();
        
        let mut rx_chan = Some(rx);
        if let Some(target) = &target_pane {
            crate::ipc::start_listener(target, tx.clone());
        }

        // Setup notify
        let mut watcher = None;
        let tx_notify = tx.clone();
        if let Ok(mut w) = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            if res.is_ok() {
                let _ = tx_notify.send("!REFRESH".to_string());
            }
        }) {
            let _ = w.watch(&root, RecursiveMode::NonRecursive);
            watcher = Some(w);
        }

        Ok(Self {
            should_quit: false,
            root,
            items,
            selected_index: 0,
            target_pane,
            rx: rx_chan,
            _watcher: watcher,
        })
    }

    pub fn refresh(&mut self) -> io::Result<()> {
        let expanded_paths: std::collections::HashSet<_> = self
            .items
            .iter()
            .filter(|item| item.is_expanded)
            .map(|item| item.path.clone())
            .collect();

        let selected_path = if !self.items.is_empty() {
            Some(self.items[self.selected_index].path.clone())
        } else {
            None
        };

        self.items = Self::read_root(&self.root)?;

        let mut i = 0;
        while i < self.items.len() {
            if expanded_paths.contains(&self.items[i].path) {
                self.items[i].is_expanded = true;
                let path = self.items[i].path.clone();
                let depth = self.items[i].depth;
                if let Ok(mut new_items) = read_dir(&path, depth + 1) {
                    let splice_idx = i + 1;
                    let mut tail = self.items.split_off(splice_idx);
                    self.items.append(&mut new_items);
                    self.items.append(&mut tail);
                }
            }
            i += 1;
        }

        if let Some(target) = selected_path {
            if let Some(idx) = self.items.iter().position(|item| item.path == target) {
                self.selected_index = idx;
            } else {
                self.selected_index = 0;
            }
        } else {
            self.selected_index = 0;
        }

        Ok(())
    }

    pub fn toggle_expand(&mut self) -> io::Result<()> {
        if self.items.is_empty() {
            return Ok(());
        }

        let item = &self.items[self.selected_index];
        if !item.is_dir {
            return Ok(());
        }

        let is_expanded = item.is_expanded;
        let depth = item.depth;
        let path = item.path.clone();

        if is_expanded {
            // Collapse
            self.items[self.selected_index].is_expanded = false;
            let mut remove_count = 0;
            for i in (self.selected_index + 1)..self.items.len() {
                if self.items[i].depth > depth {
                    remove_count += 1;
                } else {
                    break;
                }
            }
            if remove_count > 0 {
                self.items
                    .drain((self.selected_index + 1)..=(self.selected_index + remove_count));
            }
        } else {
            // Expand
            self.items[self.selected_index].is_expanded = true;
            let mut new_items = read_dir(&path, depth + 1)?;
            let splice_idx = self.selected_index + 1;
            let mut tail = self.items.split_off(splice_idx);
            self.items.append(&mut new_items);
            self.items.append(&mut tail);
            
            // Add watcher for the new directory
            if let Some(w) = &mut self._watcher {
                let _ = w.watch(&path, RecursiveMode::NonRecursive);
            }
        }
        Ok(())
    }

    pub fn expand(&mut self) -> io::Result<()> {
        if self.items.is_empty() {
            return Ok(());
        }
        let item = &self.items[self.selected_index];
        if item.is_dir && !item.is_expanded {
            self.toggle_expand()?;
        }
        Ok(())
    }

    pub fn collapse(&mut self) -> io::Result<()> {
        if self.items.is_empty() {
            return Ok(());
        }
        let item = &self.items[self.selected_index];
        if item.is_dir && item.is_expanded {
            self.toggle_expand()?;
        } else if item.depth > 0 {
            // Move up to parent
            let target_depth = item.depth - 1;
            for i in (0..self.selected_index).rev() {
                if self.items[i].depth == target_depth {
                    self.selected_index = i;
                    break;
                }
            }
        }
        Ok(())
    }

    pub fn cd_into(&mut self) -> io::Result<()> {
        if self.items.is_empty() {
            return Ok(());
        }
        let item = &self.items[self.selected_index];
        if item.is_dir {
            let new_root = item.path.clone();
            self.root = new_root.clone();
            self.items = Self::read_root(&self.root)?;
            self.selected_index = 0;
            
            // We could update the root watcher here, but relying on inbound IPC or full refresh is safer

            if let Some(target) = &self.target_pane {
                let mut cmd = std::process::Command::new("tmux");
                cmd.args([
                    "send-keys",
                    "-t",
                    target,
                    &format!("cd '{}'\n", new_root.display()),
                ]);
                let _ = cmd.output();
            }
        }
        Ok(())
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected_index + 1 < self.items.len() {
            self.selected_index += 1;
        }
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
        let mut tick_counter = 0;
        
        while !self.should_quit {
            let mut do_refresh = false;
            let mut new_path = None;

            if let Some(rx) = &self.rx {
                while let Ok(msg) = rx.try_recv() {
                    if msg == "!REFRESH" {
                        do_refresh = true;
                    } else {
                        new_path = Some(msg);
                    }
                }
            }

            if let Some(msg) = new_path {
                let new_root = PathBuf::from(msg);
                if new_root.is_dir() && new_root != self.root {
                    self.root = new_root.clone();
                    if let Ok(items) = Self::read_root(&self.root) {
                        self.items = items;
                        self.selected_index = 0;
                        
                        if let Some(w) = &mut self._watcher {
                            let _ = w.watch(&self.root, RecursiveMode::NonRecursive);
                        }
                    }
                }
            } else if do_refresh {
                let _ = self.refresh();
            }

            tick_counter += 1;
            if tick_counter >= 20 {
                tick_counter = 0;
                if let Some(target) = &self.target_pane {
                    if let Ok(output) = std::process::Command::new("tmux")
                        .args(["list-panes", "-a", "-F", "#{pane_id}"])
                        .output()
                    {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        if !stdout.contains(target) {
                            self.should_quit = true;
                        }
                    }
                }
            }

            terminal.draw(|f| ui::draw(f, self))?;

            if event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                        KeyCode::Char('c')
                            if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            self.should_quit = true;
                        }
                        KeyCode::Up | KeyCode::Char('k') => self.move_up(),
                        KeyCode::Down | KeyCode::Char('j') => self.move_down(),
                        KeyCode::Right | KeyCode::Char('l') => self.expand()?,
                        KeyCode::Left | KeyCode::Char('h') => self.collapse()?,
                        KeyCode::Enter => self.cd_into()?,
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }
}
