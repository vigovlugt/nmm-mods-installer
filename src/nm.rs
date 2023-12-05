use reqwest::Client;
use scraper::{Html, Selector};

pub async fn get_install_link(client: &Client, game: &str, mod_id: u32, file_id: u32) -> String {
    let url =
        format!("https://www.nexusmods.com/{game}/mods/{mod_id}?tab=files&file_id={file_id}&nmm=1");
    let download_page = client.get(&url).send().await.unwrap().text().await.unwrap();

    let html = Html::parse_document(&download_page);
    let selector = Selector::parse("#slowDownloadButton").unwrap();
    let el = html.select(&selector).next().unwrap();
    let attribute = el.attr(&"data-download-url");
    println!("{}", attribute.unwrap());

    return attribute.unwrap().to_string();
}
