use clap::Parser;
use cli::Args;
use config::{get_id, get_name, ModsFile};
use dialoguer::{MultiSelect, Select};
use dotenv::dotenv;
use nm::{get_install_link, get_mod_files, get_mod_name, ModFile, ModFiles};
use reqwest::{cookie::Jar, Client, ClientBuilder, Url};
use slug::slugify;
use std::{fs, process::Command, task::Wake};
use toml_edit::Document;

mod cli;
mod config;
mod nm;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let cli = Args::parse();
    let jar = Jar::default();
    let sid_develop =
        std::env::var("SID_DEVELOP").expect("No SID_DEVELOP Cookie in .env file or environment");
    jar.add_cookie_str(
        &format!("sid_develop={sid_develop}"),
        &"https://www.nexusmods.com".parse::<Url>().unwrap(),
    );
    let client = ClientBuilder::new()
        .cookie_provider(jar.into())
        .build()
        .unwrap();

    let toml_contents = fs::read_to_string("./mods.toml").expect("Could not find mods.toml");
    let mod_file: config::ModsFile = toml::from_str(&toml_contents).unwrap();
    let mut mod_file_doc = toml_contents.parse::<Document>().expect("invalid doc");

    match cli.cmd {
        cli::Commands::Install => {
            for mod_entry in mod_file.mods.iter() {
                // Install main file
                let url = get_install_link(
                    &client,
                    &get_name(&mod_file.game),
                    get_id(mod_entry.0),
                    get_id(&mod_entry.1.main_file),
                )
                .await;
                open_nxm(&url);

                // Install optional files
                if let Some(files) = &mod_entry.1.optional_files {
                    for file in files.iter() {
                        let url = get_install_link(
                            &client,
                            &get_name(&mod_file.game),
                            get_id(mod_entry.0),
                            get_id(file),
                        )
                        .await;
                        open_nxm(&url);
                    }
                }

                if let Some(files) = &mod_entry.1.misc_files {
                    let url = get_install_link(
                        &client,
                        &get_name(&mod_file.game),
                        get_id(mod_entry.0),
                        get_id(files),
                    )
                    .await;
                    open_nxm(&url);
                }
            }
        }
        cli::Commands::Add { mod_id } => {
            install_mod(&client, &mod_file, &mut mod_file_doc, mod_id).await;
        }
    }
}
async fn install_mod(
    client: &Client,
    mod_file: &ModsFile,
    mod_file_doc: &mut Document,
    mod_id: u32,
) {
    let mod_name = get_mod_name(&client, &get_name(&mod_file.game), mod_id).await;
    let files = get_mod_files(&client, &get_name(&mod_file.game), mod_id).await;

    let main_file = files.main_files[Select::new()
        .with_prompt("Select main file")
        .items(&files.main_files)
        .default(0)
        .interact()
        .unwrap()]
    .clone();

    let optional_files = if !&files.optional_files.is_empty() {
        MultiSelect::new()
            .with_prompt("Select optional files")
            .items(&files.optional_files)
            .interact()
            .unwrap()
            .iter()
            .map(|i| files.optional_files[*i].clone())
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let misc_files = if !&files.misc_files.is_empty() {
        MultiSelect::new()
            .with_prompt("Select misc files")
            .items(&files.misc_files)
            .interact()
            .unwrap()
            .iter()
            .map(|i| files.misc_files[*i].clone())
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let mut mod_table = toml_edit::table();
    mod_table["main_file"] =
        toml_edit::value(main_file.id.to_string() + "_" + &slugify(&main_file.name));

    let mut optional_files_array = toml_edit::Array::new();
    let mut misc_files_array = toml_edit::Array::new();
    for file in optional_files.iter() {
        optional_files_array.push(file.id.to_string() + "_" + &slugify(&file.name));
    }

    for file in misc_files.iter() {
        misc_files_array.push(file.id.to_string() + "_" + &slugify(&file.name));
    }

    if !optional_files_array.is_empty() {
        mod_table["optional_files"] = toml_edit::value(optional_files_array);
    }
    if !misc_files_array.is_empty() {
        mod_table["misc_files"] = toml_edit::value(misc_files_array);
    }

    let mod_ident = mod_id.to_string() + "_" + &slugify(&mod_name);
    mod_file_doc["mods"][&mod_ident] = mod_table;
    mod_file_doc["mods"][&mod_ident]
        .as_inline_table_mut()
        .map(|t| t.fmt());

    fs::write("./mods.toml", mod_file_doc.to_string()).expect("Could not write to mods.toml");
}

fn open_nxm(url: &str) -> bool {
    if let Ok(mut child) = Command::new(r"C:\Modding\MO2\nxmhandler.exe")
        .arg(url)
        .spawn()
    {
        if let Ok(status) = child.wait() {
            return status.success();
        }
    }
    false
}
