use std::time::Duration;
use tokio::{
    net::TcpStream,
    sync::mpsc,
    time::{sleep, timeout},
};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};
use futures_util::{SinkExt, StreamExt};

#[cfg(target_os = "macos")]
mod platform {
    use std::process::Command;
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateType};

    pub fn get_capslock_state() -> bool {
        let source = CGEventSource::new(CGEventSourceStateType::HIDSystemState)
            .expect("Failed to create event source");
        source.get_keyboard_state(0x39)
    }
    
    pub fn set_capslock_state(enabled: bool) {
        let script = format!(
            r#"
            ObjC.import("IOKit");
            ObjC.import("CoreServices");
            (() => {{
                var ioConnect = Ref();
                $.IOServiceOpen(
                    $.IOServiceGetMatchingService(
                        $.kIOMasterPortDefault,
                        $.IOServiceMatching($.kIOHIDSystemClass)
                    ),
                    $.mach_task_self_,
                    $.kIOHIDParamConnectType,
                    ioConnect
                );
                $.IOHIDSetModifierLockState(ioConnect, $.kIOHIDCapsLockState, {});
                $.IOServiceClose(ioConnect);
            }})();
            "#,
            if enabled { 1 } else { 0 }
        );
        
        Command::new("osascript")
            .args(&["-l", "JavaScript", "-e", &script])
            .status()
            .expect("Failed to execute osascript");
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use winapi::{
        um::winuser::{GetKeyState, keybd_event, VK_CAPITAL, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP},
        shared::windef::DWORD
    };

    pub fn get_capslock_state() -> bool {
        unsafe { (GetKeyState(VK_CAPITAL) & 0x0001) != 0 }
    }
    
    pub fn set_capslock_state(enabled: bool) {
        let current = get_capslock_state();
        if current != enabled {
            unsafe {
                keybd_event(VK_CAPITAL as u8, 0x45, KEYEVENTF_EXTENDEDKEY, 0);
                keybd_event(VK_CAPITAL as u8, 0x45, KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP, 0);
            }
        }
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use std::{
        fs,
        process::Command,
        path::Path,
    };
    use glob::glob;
    
    pub fn get_capslock_state() -> bool {
        let pattern = "/sys/class/leds/input*::capslock/brightness";
        let files: Vec<_> = glob(pattern)
            .expect("Failed to read glob pattern")
            .filter_map(Result::ok)
            .collect();
        
        if files.is_empty() {
            panic!("No CapsLock LED files found");
        }
        
        fs::read_to_string(&files[0])
            .map(|s| s.trim() == "1")
            .expect("Failed to read CapsLock state")
    }
    
    pub fn set_capslock_state(enabled: bool) {
        let current = get_capslock_state();
        if current != enabled {
            Command::new("xdotool")
                .args(&["key", "Caps_Lock"])
                .status()
                .expect("Failed to execute xdotool");
        }
    }
    
    pub fn check_dependencies() {
        if !Path::new("/usr/bin/xdotool").exists() {
            panic!("xdotool is required. Install with: sudo pacman -S xdotool");
        }
    }
}

async fn connect_websocket() -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn std::error::Error>> {
    let uri = "wss://globalcapslock.com/ws";
    let (ws_stream, _) = connect_async(uri).await?;
    Ok(ws_stream)
}

#[tokio::main]
async fn main() {
    #[cfg(target_os = "linux")]
    platform::check_dependencies();
    
    loop {
        match connect_websocket().await {
            Ok(ws_stream) => {
                println!("Connected to server");
                let (mut write, mut read) = ws_stream.split();
                
                // Send initial state
                let initial_state = platform::get_capslock_state();
                if let Err(e) = write.send(Message::Text(if initial_state { "1".into() } else { "0".into() })).await {
                    eprintln!("Failed to send initial state: {}", e);
                    continue;
                }
                
                let (tx, mut rx) = mpsc::unbounded_channel();
                
                // Spawn reader task
                let write_handle = tokio::spawn(async move {
                    while let Some(Ok(msg)) = read.next().await {
                        if let Err(e) = tx.send(msg) {
                            eprintln!("Failed to forward message: {}", e);
                            break;
                        }
                    }
                });
                
                // Spawn state checker
                let read_handle = tokio::spawn(async move {
                    let mut last_state = platform::get_capslock_state();
                    
                    loop {
                        // Check for state changes
                        let current_state = platform::get_capslock_state();
                        if current_state != last_state {
                            let msg = Message::Text(if current_state { "1".into() } else { "0".into() });
                            if let Err(e) = write.send(msg).await {
                                eprintln!("Failed to send update: {}", e);
                                break;
                            }
                            last_state = current_state;
                        }
                        
                        // Check for incoming messages
                        match timeout(Duration::from_millis(50), rx.recv()).await {
                            Ok(Some(msg)) => {
                                if let Ok(text) = msg.to_text() {
                                    let target_state = text == "1";
                                    if target_state != platform::get_capslock_state() {
                                        platform::set_capslock_state(target_state);
                                        last_state = target_state;
                                    }
                                }
                            }
                            Err(_) => (),
                            _ => (),
                        }
                        
                        sleep(Duration::from_millis(50)).await;
                    }
                });
                
                tokio::select! {
                    _ = write_handle => (),
                    _ = read_handle => (),
                }
            }
            Err(e) => {
                eprintln!("Connection error: {}. Retrying in 2 seconds...", e);
                sleep(Duration::from_secs(2)).await;
            }
        }
    }
}
