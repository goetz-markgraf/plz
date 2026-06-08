use serde::Deserialize;

static RESET: &str = "\x1b[0m";
static CYAN_BOLD: &str = "\x1b[1;36m";
static BOLD: &str = "\x1b[1m";
static GREEN_BG: &str = "\x1b[47m";

fn apply(style: &str, text: &str) -> String {
    format!("{}{}{}", style, text, RESET)
}

#[derive(Debug, Deserialize)]
pub struct Param {
    pub param: String,
    pub desc: String,
}

#[derive(Debug, Deserialize)]
pub struct Alternative {
    pub command: String,
    pub desc: String,
    #[serde(default)]
    pub params: Vec<Param>,
}

#[derive(Debug, Deserialize)]
pub struct PlzResponse {
    pub command: String,
    pub desc: String,
    #[serde(default)]
    pub params: Vec<Param>,
    #[serde(default)]
    pub alt: Vec<Alternative>,
}

/// Extracts the JSON string from the LLM response.
/// Handles two cases:
/// 1. Raw JSON (trimmed directly parseable)
/// 2. JSON wrapped in ```json ... ``` fences (with optional surrounding text)
fn extract_json(content: &str) -> &str {
    let trimmed = content.trim();

    // Look for ```json ... ``` fence
    if let Some(start) = trimmed.find("```json") {
        let after_fence = &trimmed[start + 7..];
        // skip optional newline right after the fence marker
        let json_start = after_fence.trim_start_matches('\n').trim_start_matches('\r');
        if let Some(end) = json_start.find("```") {
            return json_start[..end].trim();
        }
    }

    // Fallback: try to find a bare ``` fence without language tag
    if let Some(start) = trimmed.find("```") {
        let after_fence = &trimmed[start + 3..];
        let json_start = after_fence.trim_start_matches('\n').trim_start_matches('\r');
        if let Some(end) = json_start.find("```") {
            let candidate = json_start[..end].trim();
            if candidate.starts_with('{') {
                return candidate;
            }
        }
    }

    trimmed
}

pub fn parse_json_response(content: &str) -> Option<PlzResponse> {
    serde_json::from_str(extract_json(content)).ok()
}

pub fn format_response(content: &str) {
    match parse_json_response(content) {
        Some(resp) => {
            println!("{}", apply(CYAN_BOLD, &resp.desc));

            println!();
            println!("{}", apply(CYAN_BOLD, "Shell-Befehl:"));
            println!("{}", apply(GREEN_BG, &resp.command));

            if !resp.params.is_empty() {
                println!();
                println!("{}", apply(BOLD, "Parameter:"));
                for p in &resp.params {
                    println!("   {} — {}", apply(BOLD, &p.param), p.desc);
                }
            }

            for alt in &resp.alt {
                println!();
                println!("{}", apply(BOLD, "Alternative:"));
                println!("{}", apply(CYAN_BOLD, &alt.desc));
                println!("{}", apply(GREEN_BG, &alt.command));
                if !alt.params.is_empty() {
                    println!();
                    println!("{}", apply(BOLD, "Parameter:"));
                    for p in &alt.params {
                        println!("   {} — {}", apply(BOLD, &p.param), p.desc);
                    }
                }
            }
        }
        None => {
            println!("{}", content);
            println!("\n\x1b[1;33m{}\x1b[0m", "(Note: Could not parse JSON response.)");
        }
    }
}

pub fn extract_command(content: &str) -> Option<String> {
    parse_json_response(content).map(|r| r.command)
}

pub fn format_command_only(content: &str) {
    match extract_command(content) {
        Some(command) => println!("{}", command),
        None => {
            println!("{}", content);
            println!("\n\x1b[1;33m{}\x1b[0m", "(Note: Could not parse JSON response.)");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_json(with_alt: bool) -> String {
        if with_alt {
            r#"{
  "command": "ls -la <directory>",
  "desc": "Lists all files including hidden ones in long format.",
  "params": [
    {"param": "-l", "desc": "Long listing format"},
    {"param": "-a", "desc": "Include hidden files"},
    {"param": "<directory>", "desc": "Target directory"}
  ],
  "alt": [
    {
      "command": "find <directory> -maxdepth 1",
      "desc": "Finds all entries in a directory non-recursively.",
      "params": [
        {"param": "<directory>", "desc": "Target directory"},
        {"param": "-maxdepth 1", "desc": "Only top-level entries"}
      ]
    }
  ]
}"#.to_string()
        } else {
            r#"{
  "command": "ls -la",
  "desc": "Lists all files including hidden ones.",
  "params": [],
  "alt": []
}"#.to_string()
        }
    }

    #[test]
    fn test_parse_json_response_with_alt() {
        let result = parse_json_response(&sample_json(true));
        assert!(result.is_some(), "Expected Some, got None");
        let r = result.unwrap();
        assert_eq!(r.command, "ls -la <directory>");
        assert!(r.desc.contains("Lists all files"));
        assert_eq!(r.params.len(), 3);
        assert_eq!(r.params[0].param, "-l");
        assert_eq!(r.alt.len(), 1);
        assert_eq!(r.alt[0].command, "find <directory> -maxdepth 1");
    }

    #[test]
    fn test_parse_json_response_no_alt() {
        let result = parse_json_response(&sample_json(false));
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.command, "ls -la");
        assert!(r.alt.is_empty());
    }

    #[test]
    fn test_parse_json_response_with_fences() {
        let fenced = format!("```json\n{}\n```", sample_json(false));
        let result = parse_json_response(&fenced);
        assert!(result.is_some(), "Should parse fenced JSON");
        assert_eq!(result.unwrap().command, "ls -la");
    }

    #[test]
    fn test_parse_json_response_with_fences_and_surrounding_text() {
        let wrapped = format!("Here is the result:\n\n```json\n{}\n```\n\nHope that helps!", sample_json(true));
        let result = parse_json_response(&wrapped);
        assert!(result.is_some(), "Should parse JSON even with surrounding text");
        assert_eq!(result.unwrap().command, "ls -la <directory>");
    }

    #[test]
    fn test_parse_json_response_invalid() {
        assert!(parse_json_response("not json at all").is_none());
        assert!(parse_json_response("some text without any json").is_none());
    }

    #[test]
    fn test_parse_json_response_whitespace_trimmed() {
        let json = format!("\n  {}\n", sample_json(false));
        assert!(parse_json_response(&json).is_some());
    }

    #[test]
    fn test_extract_command_from_json() {
        let json = sample_json(false);
        assert_eq!(extract_command(&json), Some("ls -la".to_string()));
    }

    #[test]
    fn test_extract_command_invalid_json() {
        assert_eq!(extract_command("garbage"), None);
    }

    #[test]
    fn test_params_default_empty() {
        let json = r#"{"command": "echo hi", "desc": "Echoes hi."}"#;
        let result = parse_json_response(json);
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(r.params.is_empty());
        assert!(r.alt.is_empty());
    }
}
