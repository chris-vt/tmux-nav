use std::fs;
use std::io::Read;
use std::os::unix::net::UnixListener;
use std::sync::mpsc;
use std::thread;

pub fn start_listener(target_pane: &str, tx: mpsc::Sender<String>) {
    let sock_path = format!("/tmp/tmux_nav_{}.sock", target_pane);
    let _ = fs::remove_file(&sock_path);

    if let Ok(listener) = UnixListener::bind(&sock_path) {
        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        let mut buf = [0; 1024];
                        if let Ok(n) = stream.read(&mut buf) {
                            let path = String::from_utf8_lossy(&buf[..n]).trim().to_string();
                            if !path.is_empty() {
                                let _ = tx.send(path);
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }
}
