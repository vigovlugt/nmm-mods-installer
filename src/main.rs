use clap::Parser;
use cli::Args;
use config::ModsFile;
use dotenv::dotenv;
use nm::get_install_link;
use reqwest::{cookie::Jar, ClientBuilder, Url};
use std::process::Command;

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

    let mod_file = ModsFile::parse();

    match cli.cmd {
        cli::Commands::Install => {
            for mod_entry in mod_file.mods.iter() {
                let url = get_install_link(
                    &client,
                    &mod_file.game,
                    mod_entry.0.parse::<u32>().unwrap(),
                    mod_entry.1.file_id,
                )
                .await;
                open_nxm(&url);
            }
        }
    }
}

fn open_nxm(url: &str) -> bool {
    if let Ok(mut child) = Command::new(r"C:\Modding\MO2\nxmhandler.exe")
        .arg(&url)
        .spawn()
    {
        if let Ok(status) = child.wait() {
            return status.success();
        }
    }
    false
}
