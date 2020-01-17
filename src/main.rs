use clap::{App, Arg, SubCommand};
use std::collections::HashMap as Map;
use std::vec::Vec;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use std::error::Error;
use std::fmt;

#[macro_use]
extern crate serde_derive;

mod terminal;

const VERSION: &str = "0.1.0";
const USER_AGENT: &str = "gistr/0.1.0";

const GITHUB_API_URL: &str = "https://api.github.com";
// const GIT_IO_URL: &str = "https://git.io";
const GITHUB_BASE_PATH: &str = "";

// SUBCOMMANDS
const LOGIN_CMD: &str = "login";

const LIST_EXISTING_GISTS_CMD: &str = "list";
const LIST_EXISTING_GISTS_CMD_PUBLIC_ARG: &str = "public";

const CREATE_GIST_CMD: &str = "create";
const CREATE_GIST_CMD_PUBLIC_ARG: &str = "public";
const CREATE_GIST_CMD_ANONYMOUS_ARG: &str = "anonymous";
const CREATE_GIST_CMD_DESCRIPTION_ARG: &str = "description";
const CREATE_GIST_CMD_DESCRIPTION_VALUE_NAME: &str = "DESCRIPTION";
const CREATE_GIST_CMD_FILE_ARG: &str = "file";
const CREATE_GIST_CMD_FILE_ARG_VALUE_NAME: &str = "FILE(S)";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("gistr")
        .version(VERSION)
        .author("")
        .about("github gists cli tool")
        .subcommand(SubCommand::with_name(LOGIN_CMD).about("login"))
        .subcommand(
            SubCommand::with_name(CREATE_GIST_CMD)
                .about("create gist(s)")
                .arg(
                    Arg::with_name(CREATE_GIST_CMD_PUBLIC_ARG)
                        .short("p")
                        .long(CREATE_GIST_CMD_PUBLIC_ARG)
                        .help("makes gist public"),
                )
                .arg(
                    Arg::with_name(CREATE_GIST_CMD_ANONYMOUS_ARG)
                        .short("a")
                        .long(CREATE_GIST_CMD_ANONYMOUS_ARG)
                        .help("makes gist anonymous"),
                )
                .arg(
                    Arg::with_name(CREATE_GIST_CMD_DESCRIPTION_ARG)
                        .short("d")
                        .long(CREATE_GIST_CMD_DESCRIPTION_ARG)
                        .required(true)
                        .takes_value(true)
                        .multiple(false)
                        .value_name(CREATE_GIST_CMD_DESCRIPTION_VALUE_NAME)
                        .help("gist description"),
                )
                .arg(
                    Arg::with_name(CREATE_GIST_CMD_FILE_ARG)
                        .required(true)
                        .takes_value(false)
                        .multiple(true)
                        .value_name(CREATE_GIST_CMD_FILE_ARG_VALUE_NAME)
                        .help("gist file"),
                ),
        )
        .subcommand(
            SubCommand::with_name(LIST_EXISTING_GISTS_CMD)
                .about("list gists for the user")
                .arg(
                    Arg::with_name(LIST_EXISTING_GISTS_CMD_PUBLIC_ARG)
                        .short("p")
                        .long(LIST_EXISTING_GISTS_CMD_PUBLIC_ARG)
                        .help("list only public gists"),
                ),
        )
        .get_matches();

    match matches.subcommand_name() {
        Some(CREATE_GIST_CMD) => {
            let token = read_token_from_file();

            if token.is_err() {
                print!("Failed to read token file");
                return Ok(());
            }

            let token = token.unwrap();

            let public = matches
                .subcommand_matches(CREATE_GIST_CMD)
                .and_then(|sub_cmd| Some(sub_cmd.is_present(CREATE_GIST_CMD_PUBLIC_ARG)))
                .or_else(|| Some(false))
                .unwrap();
            let anonymous = matches
                .subcommand_matches(CREATE_GIST_CMD)
                .and_then(|sub_cmd| Some(sub_cmd.is_present(CREATE_GIST_CMD_ANONYMOUS_ARG)))
                .or_else(|| Some(false))
                .unwrap();
            let files_opt = matches
                .subcommand_matches(CREATE_GIST_CMD)
                .and_then(|sub_cmd| sub_cmd.values_of(CREATE_GIST_CMD_FILE_ARG));

            if files_opt.is_none() {
                print!("no file is provided");
                return Ok(());
            }

            let files = files_opt.unwrap().map(|file| String::from(file)).collect();

            let description_opt = matches
                .subcommand_matches(CREATE_GIST_CMD)
                .and_then(|sub_cmd| sub_cmd.values_of(CREATE_GIST_CMD_DESCRIPTION_ARG))
                .and_then(|mut description_values| description_values.next())
                .and_then(|s| Some(String::from(s)));

            if description_opt.is_none() {
                print!("no description is provided");
                return Ok(());
            }

            let description = description_opt.unwrap();

            let options = CreateGistOptions {
                description: description,
                public: public,
                anonymous: anonymous,
                files: files,
                access_token: token,
            };

            create_gist(&options).await?;
        }
        Some(LIST_EXISTING_GISTS_CMD) => {
            let token = read_token_from_file();

            if token.is_err() {
                print!("Failed to read token file");
                return Ok(());
            }

            let token = token.unwrap();

            let public = matches
                .subcommand_matches(LIST_EXISTING_GISTS_CMD)
                .and_then(|sub_cmd| Some(sub_cmd.is_present(LIST_EXISTING_GISTS_CMD_PUBLIC_ARG)))
                .or_else(|| Some(false))
                .unwrap();

            let options = ListAllGistsOptions {
                public: public,
                access_token: token,
            };
            list_all_gists(&options).await?;
        }
        _ => (),
    }
    Ok(())
}
#[derive(Deserialize, Debug)]
struct GistFilePayload {
    filename: String,
    raw_url: String,
    language: Option<String>,
    size: u64,
}

#[derive(Deserialize, Debug)]
struct GistPayload {
    url: String,
    forks_url: String,
    commits_url: String,
    id: String,
    public: bool,
    created_at: String,
    description: String,
    files: Map<String, GistFilePayload>,
}

#[derive(Debug)]
struct CreateGistOptions {
    description: String,
    public: bool,
    anonymous: bool,
    access_token: String,
    files: Vec<String>,
}

#[derive(Serialize, Debug)]
struct CreateGistFilePayload {
    content: String,
}

#[derive(Serialize, Debug)]
struct CreateGistPayload {
    description: String,
    public: bool,
    files: Map<String, CreateGistFilePayload>,
}

#[derive(Deserialize, Debug)]
struct CreateGistReponsePayload {
    url: String,
    html_url: String,
}

#[derive(Debug)]
struct CreateGistError {}
impl fmt::Display for CreateGistError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SuperError is here!")
    }
}
impl Error for CreateGistError {
    fn description(&self) -> &str {
        "File error"
    }
}

async fn create_gist(options: &CreateGistOptions) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!(
        "{}{}/gists?access_token={}",
        GITHUB_API_URL,
        GITHUB_BASE_PATH,
        options.access_token.as_str()
    );
    let client = reqwest::Client::new();

    let mut files: Map<String, CreateGistFilePayload> = Map::new();

    for file in &options.files {
        let path = Path::new(file.as_str());
        let file_content_opt = read_file(&path);

        match file_content_opt {
            Ok(file_content) => {
                let file_payload = CreateGistFilePayload {
                    content: file_content,
                };
                files.insert(file.clone(), file_payload);
            }
            Err(_) => return Err(Box::new(CreateGistError {})),
        }
    }

    let payload = CreateGistPayload {
        description: options.description.clone(),
        public: options.public,
        files: files,
    };

    let resp = client
        .post(url.as_str())
        .json(&payload)
        .header("user-agent", USER_AGENT)
        .send()
        .await?;

    let status = resp.status();

    if status != 201 {
        println!("status: {}", status);
        return Err(Box::new(CreateGistError {}));
    }

    let json = resp.json::<CreateGistReponsePayload>().await?;

    println!("Gist is created: {}", json.html_url.as_str());

    Ok(())
}

fn read_file(path: &Path) -> Result<String, std::io::Error> {
    let result = File::open(&path).and_then(|mut file| {
        let mut s = String::new();
        match file.read_to_string(&mut s) {
            Ok(_) => return Ok(s),
            Err(why) => return Err(why),
        }
    });
    result
}

fn write_to_file(path: &String, content: String) -> Result<(), std::io::Error> {
    let path = Path::new(path.as_str());

    let result =
        File::create(&path).and_then(|mut file| match file.write_all(content.as_bytes()) {
            Ok(_) => return Ok(()),
            Err(why) => return Err(why),
        });
    result
}

fn read_token_from_file() -> Result<String, std::io::Error> {
    let token_path = dirs::home_dir().unwrap().join(".gistr");
    read_file(&token_path).and_then(|token| Ok(String::from(token.trim())))
}

#[derive(Debug)]
struct SaveGistError {}
impl fmt::Display for SaveGistError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SuperError is here!")
    }
}
impl Error for SaveGistError {
    fn description(&self) -> &str {
        "File error"
    }
}

#[derive(Debug)]
struct ListAllGistsOptions {
    access_token: String,
    public: bool,
}

async fn list_all_gists(options: &ListAllGistsOptions) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!(
        "{}{}/gists?per_page=100?access_token={}",
        GITHUB_API_URL,
        GITHUB_BASE_PATH,
        options.access_token.as_str()
    );
    let client = reqwest::Client::new();
    let resp = client
        .get(url.as_str())
        .header("user-agent", USER_AGENT)
        .send()
        .await?;

    let gist_payloads = resp.json::<Vec<GistPayload>>().await?;
    let gist_titles: Vec<String> = gist_payloads
        .iter()
        .map(|gist| gist.description.clone())
        .collect();

    let selected_gist_opt =
        terminal::select::create_select(&gist_titles, "Please select a gist to display content")
            .and_then(|selected_index| gist_payloads.get(selected_index));

    if selected_gist_opt.is_none() {
        return Ok(());
    }

    let selected_gist = selected_gist_opt.unwrap();

    let file_titles = selected_gist
        .files
        .iter()
        .map(|(_, file)| file.filename.clone())
        .collect();

    let selected_file_opt = terminal::select::create_select(
        &file_titles,
        "Please select a gist file to display content",
    )
    .and_then(|selected_index| file_titles.get(selected_index))
    .and_then(|selected_filename| selected_gist.files.get(selected_filename));

    if selected_file_opt.is_none() {
        return Ok(());
    }
    let selected_file = selected_file_opt.unwrap();

    terminal::clear_all();

    let file_content = client
        .get(selected_file.raw_url.as_str())
        .send()
        .await?
        .text()
        .await?;

    match write_to_file(&selected_file.filename, file_content) {
        Ok(_) => return Ok(()),
        Err(_) => return Err(Box::new(SaveGistError {})),
    }
}
