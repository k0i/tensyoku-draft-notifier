#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, Write},
    path::Path,
    process,
};

use anyhow::Result;
use flexi_logger::{FileSpec, Logger, WriteMode};
use log::{error, info};
use reqwest::header::{HeaderMap, USER_AGENT};
use rev_buf_reader::RevBufReader;
use scraper::{Html, Selector};
use tauri::{api::process::restart, Manager};
use tokio::sync::{mpsc, oneshot};

use crate::commands::manual_fetch_new_log;

const USER_AGENTS :[&str;25] = [
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_11_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/53.0.2785.143 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_11_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/54.0.2840.71 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_11_6) AppleWebKit/602.1.50 (KHTML, like Gecko) Version/10.0 Safari/602.1.50",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.11; rv:49.0) Gecko/20100101 Firefox/49.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12_0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/53.0.2785.143 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12_0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/54.0.2840.71 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12_1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/54.0.2840.71 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12_1) AppleWebKit/602.2.14 (KHTML, like Gecko) Version/10.0.1 Safari/602.2.14",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12) AppleWebKit/602.1.50 (KHTML, like Gecko) Version/10.0 Safari/602.1.50",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/51.0.2704.79 Safari/537.36 Edge/14.14393",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/53.0.2785.143 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/54.0.2840.71 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/53.0.2785.143 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/54.0.2840.71 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; WOW64; rv:49.0) Gecko/20100101 Firefox/49.0",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/53.0.2785.143 Safari/537.36",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/54.0.2840.71 Safari/537.36",
    "Mozilla/5.0 (Windows NT 6.1; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/53.0.2785.143 Safari/537.36",
    "Mozilla/5.0 (Windows NT 6.1; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/54.0.2840.71 Safari/537.36",
    "Mozilla/5.0 (Windows NT 6.1; WOW64; rv:49.0) Gecko/20100101 Firefox/49.0",
    "Mozilla/5.0 (Windows NT 6.1; WOW64; Trident/7.0; rv:11.0) like Gecko",
    "Mozilla/5.0 (Windows NT 6.3; rv:36.0) Gecko/20100101 Firefox/36.0",
    "Mozilla/5.0 (Windows NT 6.3; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/53.0.2785.143 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/53.0.2785.143 Safari/537.36",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:49.0) Gecko/20100101 Firefox/49.0",
];

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
                .log_to_file(FileSpec::default().directory(log_dir).suppress_timestamp())
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

fn delta_update<P: AsRef<Path>>(mut new: Vec<String>, history_path: P) -> Result<Vec<String>> {
    let mut j = 0;
    new.reverse();
    {
        let history_file = File::open(&history_path)?;
        let rev_reader = RevBufReader::new(&history_file);
        let mut his = vec![];
        for i in rev_reader.lines() {
            if i.is_err() {
                error!("Failed to read history file: {}", i.unwrap_err());
                return Ok(new);
            }
            let line = i.unwrap();
            his.push(line);
            if his.len() == 2 {
                break;
            }
        }

        while j + 1 < new.len() && his.len() == 2 {
            if his[0] == new[j] && his[1] == new[j + 1] {
                new.truncate(j);
                break;
            }
            j += 1;
        }
    }
    let mut history_writer = OpenOptions::new().append(true).open(&history_path)?;
    for k in (0..new.len()).rev() {
        writeln!(history_writer, "{}", new[k])?;
    }
    history_writer.flush()?;
    Ok(new)
}

async fn fetch_event(id: String, tx: mpsc::Sender<Vec<String>>) -> Result<()> {
    let mut user_agent_index = 0;
    loop {
        let mut result = Vec::new();
        {
            let url = format!("https://job-draft.jp/users/{}", id);
            let mut headers = HeaderMap::new();
            headers.insert(USER_AGENT, USER_AGENTS[user_agent_index].parse()?);
            let client = reqwest::Client::new();
            let res = client
                .get(&url)
                .headers(headers)
                .send()
                .await?
                .text()
                .await?;
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
        if user_agent_index == USER_AGENTS.len() - 1 {
            user_agent_index = 0;
        } else {
            user_agent_index += 1;
        }
        tokio::time::sleep(std::time::Duration::from_secs(20)).await;
    }
}
