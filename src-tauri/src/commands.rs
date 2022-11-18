use scraper::{Html, Selector};

#[tauri::command]
pub async fn manual_fetch_new_log(id: String) -> Vec<String> {
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
