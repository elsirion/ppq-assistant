use clap::{Arg, Command};
use colored::*;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use dirs::home_dir;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{self, Write},
    path::PathBuf,
    process::{Command as ProcessCommand, Stdio},
};

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Clone, Deserialize)]
struct CodeSnippet {
    language: String,
    code: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    api_token: String,
    #[serde(default = "default_api_url")]
    api_url: String,
    #[serde(default = "default_model")]
    default_model: String,
}

#[derive(Debug)]
struct SupportedLanguage {
    name: &'static str,
    markdown_tags: &'static [&'static str],
    interpreter: &'static str,
    interpreter_flags: &'static [&'static str],
}

const SUPPORTED_LANGUAGES: &[SupportedLanguage] = &[
    SupportedLanguage {
        name: "Bash",
        markdown_tags: &["bash", "sh"],
        interpreter: "bash",
        interpreter_flags: &["-c"],
    },
    SupportedLanguage {
        name: "Python",
        markdown_tags: &["python", "python3"],
        interpreter: "python3",
        interpreter_flags: &["-c"],
    },
    SupportedLanguage {
        name: "JavaScript",
        markdown_tags: &["js", "javascript", "node"],
        interpreter: "node",
        interpreter_flags: &["-e"],
    },
    SupportedLanguage {
        name: "Ruby",
        markdown_tags: &["ruby"],
        interpreter: "ruby",
        interpreter_flags: &["-e"],
    },
    SupportedLanguage {
        name: "Perl",
        markdown_tags: &["perl"],
        interpreter: "perl",
        interpreter_flags: &["-e"],
    },
    SupportedLanguage {
        name: "PHP",
        markdown_tags: &["php"],
        interpreter: "php",
        interpreter_flags: &["-r"],
    },
];

// List of available models
const AVAILABLE_MODELS: [&str; 20] = [
    "deepseek-r1",
    "gpt-4.5-preview",
    "deepseek-chat",
    "claude-3.7-sonnet",
    "claude-3.5-sonnet",
    "gpt-4o",
    "llama-3.1-405b-instruct",
    "llama-3-70b-instruct",
    "gpt-4o-mini",
    "gemini-flash-1.5",
    "mixtral-8x7b-instruct",
    "claude-3-5-haiku-20241022:beta",
    "gemini-2.0-flash-exp",
    "grok-2",
    "qwq-32b-preview",
    "nova-pro-v1",
    "llama-3.1-nemotron-70b-instruct",
    "gemini-flash-1.5",
    "gpt-4",
    "dolphin-mixtral-8x22b",
];
const DEFAULT_MODEL: &str = "claude-3.7-sonnet";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("ppq-assistant")
        .arg(
            Arg::new("model")
                .long("model")
                .value_parser(AVAILABLE_MODELS)
                .required(false)
                .default_value(DEFAULT_MODEL),
        )
        .arg(Arg::new("prompt").num_args(1..).required(true))
        .get_matches();

    // Parse arguments to extract prompt
    let mut prompt_parts: Vec<String> = Vec::new();
    let mut reached_delimiter = false;

    if let Some(values) = matches.get_many::<String>("prompt") {
        for arg in values {
            if arg == "--" && !reached_delimiter {
                reached_delimiter = true;
                continue;
            }

            if reached_delimiter || !arg.starts_with("--") {
                prompt_parts.push(arg.to_string());
            }
        }
    }

    let prompt = prompt_parts.join(" ");
    if prompt.is_empty() {
        eprintln!("Error: No prompt provided");
        return Ok(());
    }

    // Read config file
    let config = read_config()?;

    // Get the model from arguments or use config default
    let model = matches
        .get_one::<String>("model")
        .cloned()
        .unwrap_or_else(|| config.default_model.clone());

    // Make the API request with config values
    let response = send_request_async(&config.api_token, &model, &prompt).await?;

    // Extract code snippets from the response
    let snippets = extract_code_snippets(&response);

    if snippets.is_empty() {
        println!("{}", response);

        println!("\n{}", "No executable code snippets found.".yellow());
        return Ok(());
    }

    // Display the full response first
    println!("{}", response);

    // Display and allow selection of code snippets
    if let Some(selected_snippet) = select_snippet(&snippets)? {
        execute_snippet(&selected_snippet)?;
    }

    Ok(())
}

fn read_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = get_config_path()?;

    // Create default config if file doesn't exist
    if !config_path.exists() {
        return Err(
            "Config file not found. Please create ~/.ppq/config.json with your API token".into(),
        );
    }

    let file = File::open(&config_path)
        .map_err(|_| format!("Could not open config file at {:?}", config_path))?;

    let config: Config = serde_json::from_reader(file)?;
    Ok(config)
}

fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut config_path = home_dir().ok_or("Could not find home directory")?;
    config_path.push(".ppq");
    config_path.push("config.json");
    Ok(config_path)
}

async fn send_request_async(
    api_token: &str,
    model: &str,
    prompt: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let config = read_config()?;
    let client = Client::new();
    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: prompt.to_string(),
        }],
    };

    let response = client
        .post(&config.api_url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_token))
        .json(&request)
        .send()
        .await?;

    let chat_response: ChatResponse = response.json().await?;
    Ok(chat_response.choices[0].message.content.clone())
}

fn extract_code_snippets(markdown: &str) -> Vec<CodeSnippet> {
    let code_block_regex = Regex::new(r"```(\w+)?\s*\n([\s\S]*?)\n```").unwrap();
    let mut snippets = Vec::new();

    for cap in code_block_regex.captures_iter(markdown) {
        let language = cap.get(1).map_or("text", |m| m.as_str()).to_string();
        let code = cap.get(2).map_or("", |m| m.as_str()).to_string();

        // Only include executable snippets
        if is_executable(&language) {
            snippets.push(CodeSnippet { language, code });
        }
    }

    snippets
}

fn is_executable(language: &str) -> bool {
    find_language(language).is_ok()
}

fn select_snippet(
    snippets: &[CodeSnippet],
) -> Result<Option<CodeSnippet>, Box<dyn std::error::Error>> {
    // Only show the last 10 snippets if there are more than 10
    let display_snippets = if snippets.len() > 10 {
        &snippets[snippets.len() - 10..]
    } else {
        snippets
    };

    println!("\n{}", "Select a code snippet to execute:".green().bold());

    // Display the snippets with their indices
    for (i, snippet) in display_snippets.iter().enumerate() {
        let language =
            find_language(&snippet.language).expect("only supported languages are displayed");
        println!(
            "{}: {} snippet ({} lines)",
            i.to_string().cyan().bold(),
            language.name.yellow().bold(),
            snippet.code.lines().count()
        );
        // Preview first n lines
        const PREVIEW_LINES: usize = 3;
        for line in snippet.code.lines().take(PREVIEW_LINES) {
            println!("   {}", line.trim());
        }

        if snippet.code.lines().count() > PREVIEW_LINES {
            println!("   ...");
        }

        println!();
    }

    println!(
        "Press a number key (0-9) to select, or use arrow keys and Enter. Ctrl+C or Esc to abort."
    );

    // Enable raw mode to capture keystrokes
    enable_raw_mode()?;

    let mut selected = 0;
    let mut result = None;

    loop {
        // Display selection indicator
        print!("\r");
        for i in 0..display_snippets.len() {
            if i == selected {
                print!("[{}] ", i.to_string().green().bold());
            } else {
                print!(" {}  ", i.to_string().cyan());
            }
        }
        print!("\r");
        io::stdout().flush()?;

        // Wait for a key press
        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Char('0'..='9') => {
                    let num = match code {
                        KeyCode::Char(c) => c.to_digit(10).unwrap() as usize,
                        _ => unreachable!(),
                    };
                    if num < display_snippets.len() {
                        result = Some(display_snippets[num].clone());
                        break;
                    }
                }
                KeyCode::Left if selected > 0 => selected -= 1,
                KeyCode::Right if selected < display_snippets.len() - 1 => selected += 1,
                KeyCode::Enter => {
                    result = Some(display_snippets[selected].clone());
                    break;
                }
                KeyCode::Esc | KeyCode::Char('c') if event::KeyModifiers::CONTROL.is_empty() => {
                    break
                }
                _ => {}
            }
        }
    }

    // Disable raw mode
    disable_raw_mode()?;
    println!();

    Ok(result)
}

fn find_language(language: &str) -> Result<&'static SupportedLanguage, String> {
    SUPPORTED_LANGUAGES
        .iter()
        .find(|lang| lang.markdown_tags.contains(&language))
        .ok_or_else(|| format!("Unsupported language: {}", language))
}

fn execute_snippet(snippet: &CodeSnippet) -> Result<(), Box<dyn std::error::Error>> {
    let lang = find_language(&snippet.language)?;

    println!(
        "\n{}\n",
        format!("Executing {} snippet...", lang.name).green().bold()
    );

    let output = ProcessCommand::new(lang.interpreter)
        .args(
            &lang
                .interpreter_flags
                .iter()
                .copied()
                .chain(vec![snippet.code.as_str()])
                .collect::<Vec<&str>>(),
        )
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;

    println!(
        "\n{}\n",
        if output.status.success() {
            "Execution completed successfully.".green().bold()
        } else {
            format!(
                "Execution failed with status: {}",
                output.status.code().unwrap_or(-1)
            )
            .red()
            .bold()
        }
    );

    Ok(())
}

fn default_api_url() -> String {
    "https://api.ppq.ai/chat/completions".to_string()
}

fn default_model() -> String {
    DEFAULT_MODEL.to_string()
}
