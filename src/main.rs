use clap::{App, Arg, SubCommand};
use std::collections::HashMap as Map;
use std::vec::Vec;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use std::error::Error;
use std::fmt;

use uuid::Uuid;

#[macro_use]
extern crate serde_derive;

mod terminal;

const VERSION: &str = "0.3.0";
const USER_AGENT: &str = "gistr/0.3.0";

const GITHUB_API_URL: &str = "https://api.github.com";
const GITHUB_BASE_PATH: &str = "";

// SUBCOMMANDS
const LOGIN_CMD: &str = "login";

const DISPLAY_GISTS_CMD: &str = "display";
const DISPLAY_GISTS_CMD_PUBLIC_ARG: &str = "public";

const DOWNLOAD_GISTS_CMD: &str = "download";
const DOWNLOAD_GISTS_CMD_PUBLIC_ARG: &str = "public";

const CREATE_GIST_CMD: &str = "create";
const CREATE_GIST_CMD_PUBLIC_ARG: &str = "public";
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
        .subcommand(
            SubCommand::with_name(LOGIN_CMD)
                .about("gets necessary token from github to read/write as a user and stores it in the filesystem\nit requires credentials to complete operation"),
        )
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
            SubCommand::with_name(DISPLAY_GISTS_CMD)
                .about("display gists for the user")
                .arg(
                    Arg::with_name(DISPLAY_GISTS_CMD_PUBLIC_ARG)
                        .short("p")
                        .long(DISPLAY_GISTS_CMD_PUBLIC_ARG)
                        .help("display only public gists"),
                ),
        )
        .subcommand(
            SubCommand::with_name(DOWNLOAD_GISTS_CMD)
                .about("download gists for the user")
                .arg(
                    Arg::with_name(DOWNLOAD_GISTS_CMD_PUBLIC_ARG)
                        .short("p")
                        .long(DOWNLOAD_GISTS_CMD_PUBLIC_ARG)
                        .help("download only public gists"),
                ),
        )
        .get_matches();

    match matches.subcommand_name() {
        Some(CREATE_GIST_CMD) => {
            let token = read_token_from_file();

            if token.is_err() {
                println!("Failed to read token file.\nPlease login using the login subcommand or create a token file at {} with a valid token in it.", get_token_path().to_str().unwrap());
                return Ok(());
            }

            let token = token.unwrap();

            let public = matches
                .subcommand_matches(CREATE_GIST_CMD)
                .and_then(|sub_cmd| Some(sub_cmd.is_present(CREATE_GIST_CMD_PUBLIC_ARG)))
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
                files: files,
                access_token: token,
            };

            create_gist(&options).await?;
        }
        Some(DISPLAY_GISTS_CMD) => {
            let token = read_token_from_file();

            if token.is_err() {
                println!("Failed to read token file.\nPlease login using the login subcommand or create a token file at {} with a valid token in it.", get_token_path().to_str().unwrap());
                return Ok(());
            }

            let token = token.unwrap();

            let public = matches
                .subcommand_matches(DISPLAY_GISTS_CMD)
                .and_then(|sub_cmd| Some(sub_cmd.is_present(DISPLAY_GISTS_CMD_PUBLIC_ARG)))
                .or_else(|| Some(false))
                .unwrap();

            let options = GetGistsOptions {
                public: public,
                access_token: token,
            };
            display_gists(&options).await?;
        }
        Some(DOWNLOAD_GISTS_CMD) => {
            let token = read_token_from_file();

            if token.is_err() {
                println!("Failed to read token file.\nPlease login using the login subcommand or create a token file at {} with a valid token in it.", get_token_path().to_str().unwrap());
                return Ok(());
            }

            let token = token.unwrap();

            let public = matches
                .subcommand_matches(DOWNLOAD_GISTS_CMD)
                .and_then(|sub_cmd| Some(sub_cmd.is_present(DOWNLOAD_GISTS_CMD_PUBLIC_ARG)))
                .or_else(|| Some(false))
                .unwrap();

            let options = GetGistsOptions {
                public: public,
                access_token: token,
            };
            write_gists_to_file(&options).await?;
        }
        Some(LOGIN_CMD) => {
            let username_opt = terminal::input::ask_input("Please enter your username");

            if username_opt.is_none() {
                print!("no username is provided");
                return Ok(());
            }

            let username = username_opt.unwrap();

            let password_opt = terminal::password::ask_password("Please enter your password");

            if password_opt.is_none() {
                print!("no password is provided");
                return Ok(());
            }

            let password = password_opt.unwrap();

            let uses_two_factor_auth =
                terminal::input::ask_yes_no_question("Do you use two factor authentication?");

            if uses_two_factor_auth.is_none() {
                return Ok(());
            }
            let mut two_factor_code_opt = None;
            if uses_two_factor_auth.unwrap() {
                two_factor_code_opt =
                    terminal::input::ask_input("Please enter two factor authentication code")
            }

            login(&LoginOptions {
                username: username,
                password: password,
                two_factor_code: two_factor_code_opt,
            })
            .await?;
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

impl GistPayload {
    fn get_file_payload(&mut self, filename: &str) -> Option<GistFilePayload> {
        return self.files.remove(filename);
    }
}

#[derive(Debug)]
struct CreateGistOptions {
    description: String,
    public: bool,
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
    let url = format!("{}{}/gists", GITHUB_API_URL, GITHUB_BASE_PATH);
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
        .header(
            "Authorization",
            format!("token {}", options.access_token.as_str()),
        )
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

fn get_token_path() -> std::path::PathBuf {
    dirs::home_dir().unwrap().join(".gistr")
}

fn read_token_from_file() -> Result<String, std::io::Error> {
    read_file(&get_token_path()).and_then(|token| Ok(String::from(token.trim())))
}

fn write_token_to_file(token: String) -> Result<(), std::io::Error> {
    write_to_file(&String::from(get_token_path().to_str().unwrap()), token)
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
struct GetGistsOptions {
    access_token: String,
    public: bool,
}

async fn select_file_from_gists(
    options: &GetGistsOptions,
) -> Result<Option<GistFilePayload>, Box<dyn std::error::Error>> {
    let url = format!("{}{}/gists?per_page=100", GITHUB_API_URL, GITHUB_BASE_PATH);
    let client = reqwest::Client::new();
    let resp = client
        .get(url.as_str())
        .header("user-agent", USER_AGENT)
        .header(
            "Authorization",
            format!("token {}", options.access_token.as_str()),
        )
        .send()
        .await?;

    let mut gist_payloads = resp.json::<Vec<GistPayload>>().await?;
    let gist_titles: Vec<String> = gist_payloads
        .iter()
        .map(|gist| {
            if !gist.description.is_empty() {
                return gist.description.clone();
            }
            return match gist.files.iter().nth(0) {
                Some((filename, _)) => filename.clone(),
                _ => String::from(""),
            };
        })
        .collect();

    let selected_gist_opt =
        terminal::select::create_select(&gist_titles, "Please select a gist to display content")
            .and_then(|selected_index| Some(gist_payloads.remove(selected_index)));

    if selected_gist_opt.is_none() {
        return Ok(None);
    }

    let mut selected_gist = selected_gist_opt.unwrap();

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
    .and_then(|selected_filename| selected_gist.get_file_payload(selected_filename))
    .and_then(|selected_file| Some(selected_file));
    terminal::clear_all();

    return Ok(selected_file_opt);
}

async fn download_gist(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let file_content = client.get(url).send().await?.text().await?;

    return Ok(file_content);
}

async fn display_gists(options: &GetGistsOptions) -> Result<(), Box<dyn std::error::Error>> {
    let file = select_file_from_gists(options).await?;

    match file {
        Some(file) => {
            let file_content = download_gist(file.raw_url.as_str()).await?;
            println!("{}", file_content);
        }
        _ => {}
    }
    Ok(())
}

async fn write_gists_to_file(options: &GetGistsOptions) -> Result<(), Box<dyn std::error::Error>> {
    let file = select_file_from_gists(options).await?;

    if file.is_none() {
        return Ok(());
    }
    let file = file.unwrap();
    let file_content = download_gist(file.raw_url.as_str()).await?;
    match write_to_file(&file.filename, file_content) {
        Ok(_) => {
            println!("{} is saved.", &file.filename);
            return Ok(());
        }
        Err(_) => return Err(Box::new(SaveGistError {})),
    }
}

#[derive(Debug)]
struct LoginOptions {
    username: String,
    password: String,
    two_factor_code: Option<String>,
}

#[derive(Serialize, Debug)]
struct LoginRequestPayload {
    scopes: Vec<String>,
    note: String,
    note_url: String,
}

#[derive(Deserialize, Debug)]
struct LoginResponsePayload {
    id: u64,
    url: String,
    token: String,
    hashed_token: String,
}

#[derive(Debug)]
struct LoginError {}
impl fmt::Display for LoginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LoginError")
    }
}
impl Error for LoginError {
    fn description(&self) -> &str {
        "Authentication Error"
    }
}

async fn login(options: &LoginOptions) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("{}{}/authorizations", GITHUB_API_URL, GITHUB_BASE_PATH);
    let client = reqwest::Client::new();

    let payload = LoginRequestPayload {
        scopes: vec![String::from("gist")],
        note: String::from(format!("gistr cli tool {}", Uuid::new_v4())),
        note_url: String::from("https://github.com/alicanerdogan/gistr"),
    };

    let req = client
        .post(url.as_str())
        .basic_auth(options.username.as_str(), Some(options.password.as_str()))
        .json(&payload)
        .header("user-agent", USER_AGENT)
        .header("content-type", "application/json");

    let resp = match &options.two_factor_code {
        Some(code) => req.header("X-GitHub-OTP", code.as_str()).send().await?,
        None => req.send().await?,
    };

    let status = resp.status();
    println!("{:?}", status);

    if status == 401 {
        println!("Failed to authenticate. Please retry to login once again.");
        return Ok(());
    }

    if status != 201 {
        return Err(Box::new(LoginError {}));
    }

    let payload = resp.json::<LoginResponsePayload>().await?;

    match write_token_to_file(payload.token) {
        Ok(_) => Ok(()),
        _ => Err(Box::new(LoginError {})),
    }
}
