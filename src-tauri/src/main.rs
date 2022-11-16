#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use scraper::{Html, Selector};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![fetch_event])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn fetch_event(id: String) -> Vec<String> {
    let url = format!("https://job-draft.jp/users/{}", id);
    let res = reqwest::get(url).await.unwrap().text().await.unwrap();
    let doc = Html::parse_document(&res);
    let s = Selector::parse("ul.c-timeline--activity-list").unwrap();
    let mut result = Vec::new();
    doc.select(&s).for_each(|e| {
        result.push(format!("{:?}", e.text().collect::<Vec<_>>()));
    });
    result
}
