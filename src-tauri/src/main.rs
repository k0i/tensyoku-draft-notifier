#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::Path,
    process,
};

use anyhow::Result;
use flexi_logger::{FileSpec, Logger, WriteMode};
use log::{error, info};
use scraper::{Html, Selector};
use tauri::{api::process::restart, Manager};
use tokio::sync::{mpsc, oneshot};

use crate::commands::manual_fetch_new_log;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct FetchNewLogPayload {
    logs: Vec<String>,
}

fn is_initial_boot<T: AsRef<Path>>(conf_path: T) -> bool {
    fs::metadata(conf_path).is_err()
}

fn main() -> Result<()> {
    let (tx, mut rx) = mpsc::channel(1);
    let (sigkill_tx, mut sigkill_rx) = oneshot::channel::<usize>();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![manual_fetch_new_log])
        .setup(move |app| {
            let h = app.handle();
            // panic if app data dir is not available
            if app.path_resolver().app_data_dir().is_none() {
                panic!("App data dir is not available");
            }
            let mut log_dir = app.path_resolver().app_data_dir().unwrap();
            log_dir.push("tensyoku-scraping");
            let _logger = Logger::try_with_env_or_str("info")?
                .log_to_file(FileSpec::default().directory(log_dir))
                .write_mode(WriteMode::BufferAndFlush)
                .start()?;
            // defining required path
            let mut user_id_path = app.path_resolver().app_data_dir().unwrap();
            user_id_path.push("tensyoku-scraping/user_id");
            let mut logs_path = app.path_resolver().app_data_dir().unwrap();
            logs_path.push("tensyoku-scraping/logs");

            // initial boot
            if is_initial_boot(&user_id_path) {
                // listening a user_id input event from frontend,  write it to file and restart the app to start scraping
                app.listen_global("input_user_id", move |event| {
                    if let Err(e) = File::create(&logs_path) {
                        error!("Failed to create logs file: {}", e);
                    };
                    if let Ok(mut f) = File::create(&user_id_path) {
                        if let Err(e) = write!(f, "{}", event.payload().unwrap_or("unknown")) {
                            error!("Failed to write user_id to file: {}", e);
                        };
                    } else {
                        error!("Failed to create user_id file");
                    };
                    restart(&h.env());
                });
                // early return because we can't start scraping without user_id
                return Ok(());
            }

            let mut user_id = match fs::read_to_string(&user_id_path) {
                Ok(s) => {
                    info!("user_id: {}", s);
                    s
                }
                Err(e) => {
                    error!("Failed to read user_id file: {}", e);
                    sigkill_tx.send(1).unwrap();
                    return Ok(());
                }
            };
            user_id = user_id[1..user_id.len() - 1].to_string();
            // scraping loop
            tauri::async_runtime::spawn(async move {
                if let Err(e) = fetch_event(user_id, tx).await {
                    error!("Failed to fetch event: {}", e);
                    sigkill_tx.send(1).unwrap();
                };
            });
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::select! {
                       logs = rx.recv() => {
                          if let Some(new_logs) = logs {
                             let logs = match delta_update(new_logs,&logs_path){
Ok(l) => l,
Err(e) => {
    error!("Failed to update logs: {}", e);
    vec![]
                             }
                             };
                             if logs.is_empty() {
                                continue;
                             }
                             h.emit_all("fetch_new_log",FetchNewLogPayload{logs}).unwrap();
                          }
                       },
                       sig = &mut sigkill_rx => {
                           error!("sigkill received: {}", sig.unwrap());
                           process::exit(1);
                       }
                    }
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    Ok(())
}

fn delta_update<P: AsRef<Path>>(new: Vec<String>, history_path: P) -> Result<Vec<String>> {
    let mut j = 0;
    {
        let mut history_file = File::open(&history_path)?;
        let history_reader = BufReader::new(&mut history_file);
        for i in history_reader.lines() {
            if i.is_err() {
                error!("Failed to read history file: {}", i.unwrap_err());
                return Ok(new);
            }
            let line = i.unwrap();
            while j < new.len() {
                if new[j] != line {
                    break;
                }
                j += 1;
            }
        }
    }
    let mut history_writer = OpenOptions::new().append(true).open(&history_path)?;
    for k in j..new.len() {
        writeln!(history_writer, "{}", new[k])?;
    }
    history_writer.flush()?;
    Ok(new[j..].to_vec())
}

async fn fetch_event(id: String, tx: mpsc::Sender<Vec<String>>) -> Result<()> {
    loop {
        let mut result = Vec::new();
        {
            let url = format!("https://job-draft.jp/users/{}", id);
            let res = reqwest::get(url).await?.text().await?;
            let doc = Html::parse_document(&res);
            let s = Selector::parse("ul.c-timeline--activity-list")
                .expect("ul.c-timeline--activity-list element not found: maybe html changed");
            let li_selector =
                Selector::parse("li").expect("li element not found: maybe html changed");
            for element in doc.select(&s) {
                for li in element.select(&li_selector) {
                    let text = li.text().collect::<Vec<_>>();
                    result.push(text.join(""));
                }
            }
        }
        result.reverse();
        if let Err(e) = tx.send(result).await {
            error!("error sending fetched result: {:?}", e);
        }
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    }
}
