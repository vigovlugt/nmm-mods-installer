use std::fmt::Display;

use reqwest::Client;
use scraper::{Html, Selector};

pub async fn get_install_link(client: &Client, game: &str, mod_id: u32, file_id: u32) -> String {
    let url =
        format!("https://www.nexusmods.com/{game}/mods/{mod_id}?tab=files&file_id={file_id}&nmm=1");
    let download_page = client.get(&url).send().await.unwrap().text().await.unwrap();

    let html = Html::parse_document(&download_page);
    let selector = Selector::parse("#slowDownloadButton").unwrap();
    let el = html.select(&selector).next().unwrap();
    let attribute = el.attr("data-download-url");
    println!("{}", attribute.unwrap());

    attribute.unwrap().to_string()
}

pub enum ModDependency {
    Mod(u32),
    External(String),
}

pub async fn get_mod_dependencies(
    client: &Client,
    game_id: u32,
    file_id: u32,
) -> Vec<ModDependency> {
    let url = format!("https://www.nexusmods.com/Core/Libs/Common/Widgets/ModRequirementsPopUp?id={file_id}&game_id={game_id}&nmm=1");
    let download_page = client.get(&url).send().await.unwrap().text().await.unwrap();

    let html = Html::parse_fragment(&download_page);
    let selector = Selector::parse("widget-mod-requirements ul li a").unwrap();
    let mut dependencies = Vec::new();
    for el in html.select(&selector) {
        let attribute = el.attr("href").unwrap();
        if attribute.starts_with("https://www.nexusmods.com/skyrimspecialedition/mods/") {
            let mod_id = attribute.split('/').last().unwrap().parse::<u32>().unwrap();
            dependencies.push(ModDependency::Mod(mod_id));
        } else {
            dependencies.push(ModDependency::External(attribute.to_string()));
        }
    }

    dependencies
}

#[derive(Debug, Clone)]
pub struct ModFile {
    pub id: u32,
    pub name: String,
    pub description: String,
}

impl Display for ModFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{name}   {description}",
            name = self.name,
            description = self.description
        )
    }
}

#[derive(Debug)]
pub struct ModFiles {
    pub main_files: Vec<ModFile>,
    pub optional_files: Vec<ModFile>,
    pub misc_files: Vec<ModFile>,
}

pub async fn get_mod_name(client: &Client, game: &str, mod_id: u32) -> String {
    let url = format!(
        "https://www.nexusmods.com/{game}/mods/{mod_id}",
        game = game,
        mod_id = mod_id
    );
    let download_page = client.get(&url).send().await.unwrap().text().await.unwrap();

    let html = Html::parse_document(&download_page);
    let selector = Selector::parse("h1").unwrap();
    let el = html.select(&selector).next().unwrap();
    let attribute = el.text().next().unwrap();

    attribute.to_string()
}

pub async fn get_mod_files(client: &Client, game: &str, mod_id: u32) -> ModFiles {
    let url = format!("https://www.nexusmods.com/{game}/mods/{mod_id}?tab=files");
    let download_page = client.get(&url).send().await.unwrap().text().await.unwrap();

    let html = Html::parse_document(&download_page);
    let main_files_selector = Selector::parse("#file-container-main-files dl").unwrap();
    let optional_files_selector = Selector::parse("#file-container-optional-files dl").unwrap();
    let misc_files_selector = Selector::parse("#file-container-miscellaneous-files dl").unwrap();

    fn get_mod_file(el: scraper::ElementRef) -> ModFile {
        let dt = el.select(&Selector::parse("dt").unwrap()).next().unwrap();
        let name = dt.attr("data-name").unwrap().to_string();
        let id = dt.attr("data-id").unwrap().parse::<u32>().unwrap();

        let description = el
            .select(&Selector::parse(".files-description > *").unwrap())
            .flat_map(|el| el.text())
            .collect::<String>();

        ModFile {
            id,
            name,
            description,
        }
    }

    let main_files = html
        .select(&main_files_selector)
        .map(get_mod_file)
        .collect::<Vec<_>>();
    let optional_files = html
        .select(&optional_files_selector)
        .map(get_mod_file)
        .collect::<Vec<_>>();
    let misc_files = html
        .select(&misc_files_selector)
        .map(get_mod_file)
        .collect::<Vec<_>>();

    ModFiles {
        main_files,
        optional_files,
        misc_files,
    }
}
