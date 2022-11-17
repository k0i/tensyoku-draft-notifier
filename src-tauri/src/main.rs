#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::Path,
};

use scraper::{Html, Selector};
use tauri::{api::process::restart, Manager};
use tokio::sync::{mpsc, oneshot};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct FetchNewLogPayload {
    logs: Vec<String>,
}

fn is_initial_boot<T: AsRef<Path>>(conf_path: T) -> bool {
    !fs::metadata(conf_path).is_ok()
}

fn main() {
    let (tx, mut rx) = mpsc::channel(1);
    let (err_tx, mut err_rx) = oneshot::channel::<String>();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![manual_fetch_new_log])
        .setup(move |app| {
            let h = app.handle();
            let mut user_id_path = app.path_resolver().app_data_dir().expect("Could not find app data directory");
            user_id_path.push("tensyoku-scraping/user_id");
            let mut logs_path = app.path_resolver().app_data_dir().expect("Could not find app data directory");
            logs_path.push("tensyoku-scraping/logs");
            app.listen_global("input_user_id", move|event| {
               if is_initial_boot(&user_id_path){
                File::create(&logs_path).expect("Could not create file");
                let mut f = File::create(&user_id_path).expect("Could not create file");
                write!(f, "{}", event.payload().unwrap_or("unknown")).expect("Could not write to file");
                restart(&h.env());}
            });
let mut conf_path = app.path_resolver().app_data_dir().expect("Could not find app data directory");
            conf_path.push("tensyoku-scraping/user_id");

            if is_initial_boot(&conf_path){
                return Ok(());
            }
            let mut user_id = fs::read_to_string(conf_path).expect("Could not read user id");
            user_id = user_id[1..user_id.len()-1].to_string();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = fetch_event(user_id, tx).await {
                    err_tx.send(e.to_string()).expect("failed to send logs");
                };
            });
            let h = app.handle();
            let mut logs_path = app.path_resolver().app_data_dir().expect("Could not find app data directory");
            logs_path.push("tensyoku-scraping/logs");
            tauri::async_runtime::spawn(async move {
                loop{
                tokio::select! {
                logs =rx.recv() => {
                                            if let Some(logs) = logs {
                                                let new_log = delta_update(logs,&logs_path);
                                                if new_log.is_empty() {
                                                    continue;
                                                }
                                            h.emit_all("fetch_new_log",FetchNewLogPayload{logs:new_log}).unwrap();
                                            }
                                        },
                                        err = &mut err_rx => {
                                            eprintln!("Error: {}", err.unwrap());
                                        }
                                    }
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn delta_update<P: AsRef<Path>>(new: Vec<String>, history_path: P) -> Vec<String> {
    let mut j = 0;
    {
        let mut history_file = File::open(&history_path).expect("Could not open history file");
        let history_reader = BufReader::new(&mut history_file);
        for i in history_reader.lines() {
            if i.is_err() {
                return new;
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
    let mut history_writer = OpenOptions::new()
        .append(true)
        .open(&history_path)
        .expect("Could not open history file");
    for k in j..new.len() {
        writeln!(history_writer, "{}", new[k]).expect("Could not write to file");
    }
    history_writer.flush().expect("Could not flush");
    new[j..].to_vec()
}

async fn fetch_event(id: String, tx: mpsc::Sender<Vec<String>>) -> Result<(), String> {
    loop {
        let mut result = Vec::new();
        {
            let url = format!("https://job-draft.jp/users/{}", id);
            let res = reqwest::get(url)
                .await
                .expect("reqwest error: user_id is invalid")
                .text()
                .await
                .expect("response parse error");
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
            eprintln!("error sending: {:?}", e);
        }
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
}

#[tauri::command]
async fn manual_fetch_new_log(id: String) -> Vec<String> {
    let mut result = Vec::new();
    let url = format!("https://job-draft.jp/users/{}", id);
    let res = reqwest::get(url)
        .await
        .expect("reqwest error: user_id is invalid")
        .text()
        .await
        .expect("response parse error");
    let doc = Html::parse_document(&res);
    let s = Selector::parse("ul.c-timeline--activity-list")
        .expect("ul.c-timeline--activity-list element not found: maybe html changed");
    let li_selector = Selector::parse("li").expect("li element not found: maybe html changed");
    for element in doc.select(&s) {
        for li in element.select(&li_selector) {
            let text = li.text().collect::<Vec<_>>();
            result.push(text.join(""));
        }
    }
    result
}
