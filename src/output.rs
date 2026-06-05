use std::borrow::Cow;

static RESET: &str = "\x1b[0m";
static CYAN_BOLD: &str = "\x1b[1;36m";
static BOLD: &str = "\x1b[1m";
static GREEN_BG: &str = "\x1b[1;42m";

fn apply(style: &str, text: &str) -> String {
    format!("{}{}{}", style, text, RESET)
}

fn strip_headings(text: &str) -> Cow<'_, str> {
    if text.starts_with("### ") || text.starts_with("###") {
        return Cow::Owned(text.trim_start_matches('#').trim_start().trim().to_string());
    } else if text.starts_with("## ") || text.starts_with("##") {
        return Cow::Owned(text.trim_start_matches('#').trim_start().trim().to_string());
    } else if text.starts_with("# ") || text.starts_with("#") {
        return Cow::Owned(text.trim_start_matches('#').trim_start().trim().to_string());
    }
    Cow::Borrowed(text)
}

fn strip_list_markers(text: &str) -> Cow<'_, str> {
    let trimmed = text.trim_start();
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
        let rest = &trimmed[2..];
        return Cow::Owned(rest.to_string());
    }
    Cow::Borrowed(text)
}

fn strip_inline_code(text: &str) -> Cow<'_, str> {
    let mut output = String::new();
    let mut in_backtick = false;
    for ch in text.chars() {
        if ch == '`' {
            in_backtick = !in_backtick;
            output.push(if in_backtick { '[' } else { ']' });
        } else {
            output.push(ch);
        }
    }
    if in_backtick {
        // Unclosed backtick, remove all
        Cow::Owned(text.replace('`', ""))
    } else {
        Cow::Owned(output)
    }
}

fn strip_bold_italic(text: &str) -> Cow<'_, str> {
    let mut result = text.to_string();
    result = result.replace("**", "").replace("__", "");
    result = result.replace('*', "").replace('_', "");
    Cow::Owned(result)
}

/// Strips markdown from a line: bold, italic, inline code, headings, list markers
fn full_strip_markdown(text: &str) -> String {
    let line = strip_list_markers(text);
    let line = strip_headings(line.as_ref());
    let line = strip_inline_code(line.as_ref());
    let line = strip_bold_italic(line.as_ref());
    line.trim().to_string()
}

fn is_horizontal_rule(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed == "---" || trimmed == "***" || trimmed == "___"
}

const SHELL_LANGUAGES: &[&str] = &[
    "bash", "sh", "shell", "zsh", "fish", "csh", "tcsh", "ksh",
    "pwsh", "powershell", "cmd", "bat", "ps1", "nu", "nushell",
];

fn is_shell_language(lang: &str) -> bool {
    SHELL_LANGUAGES.contains(&lang.to_lowercase().as_str())
}

/// Returns (description_text, command, param_lines) if a shell command block is found.
/// Parses line-by-line with a state machine to handle inline backticks correctly.
fn parse_llm_response(content: &str) -> Option<(String, String, Vec<String>)> {
    #[derive(PartialEq)]
    enum Section { Before, InCodeBlock, AfterCommand }

    let mut description = String::new();
    let mut param_lines: Vec<String> = Vec::new();
    let mut command: Option<String> = None;
    let mut code_block_lines: Vec<String> = Vec::new();
    let mut section = Section::Before;
    let mut in_params = false;

    for line in content.lines() {
        match section {
            Section::Before | Section::AfterCommand => {
                if line.trim_start().starts_with("```") {
                    let lang = line.trim_start().trim_start_matches('`').trim().to_string();
                    if command.is_none() && is_shell_language(&lang) {
                        code_block_lines.clear();
                        section = Section::InCodeBlock;
                    }
                    // Non-shell code blocks (or blocks after the command) are skipped
                    continue;
                }

                if is_horizontal_rule(line) {
                    continue;
                }

                let stripped = full_strip_markdown(line);
                let trimmed = stripped.trim();

                if trimmed.is_empty() {
                    continue;
                }

                let lower = trimmed.to_lowercase();
                let is_param_header = lower == "parameter:" || lower == "parameter"
                    || lower == "parameters:" || lower == "parameters"
                    || lower.starts_with("parameter") && lower.ends_with(':');

                if is_param_header {
                    in_params = true;
                    continue;
                }

                if in_params {
                    param_lines.push(stripped);
                } else if section == Section::Before {
                    if !description.is_empty() {
                        description.push(' ');
                    }
                    description.push_str(trimmed);
                }
            }
            Section::InCodeBlock => {
                if line.trim_start().starts_with("```") {
                    // End of code block
                    command = Some(code_block_lines.join("\n"));
                    section = Section::AfterCommand;
                } else {
                    code_block_lines.push(line.to_string());
                }
            }
        }
    }

    let cmd = command?;
    if cmd.trim().is_empty() {
        return None;
    }

    description = description.split_whitespace().collect::<Vec<_>>().join(" ");

    Some((description, cmd.trim().to_string(), param_lines))
}

pub fn format_response(content: &str) {
    if let Some((description, command, param_lines)) = parse_llm_response(content) {
        if !description.is_empty() {
            println!("{}", apply(CYAN_BOLD, &description));
            println!();
        }

        if !param_lines.is_empty() {
            println!("{}", apply(BOLD, "Parameter:"));
            for param in &param_lines {
                println!("   - {}", param);
            }
            println!();
        }

        println!("{}", apply(CYAN_BOLD, "Shell-Befehl:"));
        println!("{}", apply(GREEN_BG, &command));
    } else {
        println!("{}", content);
        println!("\n\x1b[1;33m{}\x1b[0m", "(Note: No executable command found in the response.)");
    }
}

pub fn extract_command(content: &str) -> Option<String> {
    parse_llm_response(content).map(|(_, cmd, _)| cmd)
}

pub fn format_command_only(content: &str) {
    if let Some(command) = extract_command(content) {
        println!("{}", command);
    } else {
        println!("{}", content);
        println!("\n\x1b[1;33m{}\x1b[0m", "(Note: No executable command found in the response.)");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // strip_headings tests
    #[test]
    fn test_strip_headings_removes_hashes() {
        assert_eq!(strip_headings("### Beschreibung"), "Beschreibung");
        assert_eq!(strip_headings("# Titel mit mehreren Wörtern"), "Titel mit mehreren Wörtern");
        assert_eq!(strip_headings("## Parameter"), "Parameter");
    }

    #[test]
    fn test_strip_headings_no_hashes() {
        assert_eq!(strip_headings("Plain text"), "Plain text");
    }

    // strip_list_markers tests
    #[test]
    fn test_strip_list_markers_removes_dash() {
        assert_eq!(strip_list_markers("- item text"), "item text");
    }

    #[test]
    fn test_strip_list_markers_removes_asterisk() {
        assert_eq!(strip_list_markers("* item text"), "item text");
    }

    #[test]
    fn test_strip_list_markers_no_marker() {
        assert_eq!(strip_list_markers("Plain text"), "Plain text");
    }

    // strip_inline_code tests
    #[test]
    fn test_strip_inline_code_replaces_backticks() {
        assert_eq!(strip_inline_code("`git remote`"), "[git remote]");
    }

    #[test]
    fn test_strip_inline_code_removes_unclosed() {
        let result = strip_inline_code("`unclosed");
        assert!(!result.contains('`'));
    }

    #[test]
    fn test_strip_inline_code_multiple() {
        let result = strip_inline_code("`a` and `b`");
        assert_eq!(result, "[a] and [b]");
    }

    // strip_bold_italic tests
    #[test]
    fn test_strip_bold_italic_bold() {
        assert_eq!(strip_bold_italic("**Befehl:**"), "Befehl:");
        assert_eq!(strip_bold_italic("__text__"), "text");
    }

    #[test]
    fn test_strip_bold_italic_italic() {
        assert_eq!(strip_bold_italic("*text*"), "text");
        assert_eq!(strip_bold_italic("_text_"), "text");
    }

    // full_strip_markdown tests
    #[test]
    fn test_full_strip_marks() {
        let result = full_strip_markdown("**Befehl:** *`git remote`* – text");
        assert!(result.contains("Befehl:"));
        assert!(!result.contains('*'));
        assert!(!result.contains('_'));
        assert!(!result.contains('`'));
    }

    #[test]
    fn test_full_strip_hrule() {
        assert!(is_horizontal_rule("---"));
        assert!(is_horizontal_rule("***"));
        assert!(is_horizontal_rule("___"));
        assert!(!is_horizontal_rule("text"));
    }

    // extract_command tests
    #[test]
    fn test_extract_command_bash_block() {
        let content = "The command you want is:\n\n```bash\nls -la\n```\n\nThis lists files.";
        assert_eq!(extract_command(content), Some("ls -la".to_string()));
    }

    #[test]
    fn test_extract_command_zsh_block() {
        let content = "```zsh\nls -la\n```";
        assert_eq!(extract_command(content), Some("ls -la".to_string()));
    }

    #[test]
    fn test_extract_command_no_block() {
        let content = "No code here, just text.";
        assert_eq!(extract_command(content), None);
    }

    #[test]
    fn test_extract_command_unknown_block() {
        let content = "```python\nprint('hello')\n```";
        assert_eq!(extract_command(content), None);
    }

    #[test]
    fn test_extract_command_multiline() {
        let content = "```bash\nfor i in {1..10}\ndo\n    echo $i\ndone\n```";
        assert_eq!(extract_command(content), Some("for i in {1..10}\ndo\n    echo $i\ndone".to_string()));
    }

    // parse_llm_response tests
    #[test]
    fn test_parse_response_basic() {
        // Description + Befehl: header + code block + Parameter section
        let content = "Der Befehl gibt die URL des Remote-Repositories aus.\n\n**Befehl:**\n```zsh\ngit remote get-url origin\n```\n\n**Parameter:**\n* `git remote`: Der Haupt\n* `get-url`: Fragt die URL ab";
        let result = parse_llm_response(content);
        assert!(result.is_some(), "parse_llm_response returned None");
        let r = result.unwrap();
        // "Befehl:" is stripped as a header
        assert!(r.0.contains("Befehl gibt die URL des Remote-Repositories aus"), 
                  "Expected description text, got: {:?}", r.0);
        assert_eq!(r.1, "git remote get-url origin");
        assert!(!r.2.is_empty(), "Expected at least one parameter line, got: {:?}", r.2);
    }

    #[test]
    fn test_parse_response_no_description() {
        let content = "```bash\nls -la\n```";
        let result = parse_llm_response(content);
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(r.0.trim().is_empty());
        assert_eq!(r.1, "ls -la");
    }

    #[test]
    fn test_parse_response_no_params() {
        // Note: "Befehl:" header is skipped, so only "Dieser listet" remains
        let content = "Dieser Befehl listet alle Dateien auf.\n\n**Befehl:**\n```bash\nls -la\n```";
        let result = parse_llm_response(content);
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(r.0.contains("Dieser"));
        assert!(r.0.contains("listet"));
        assert!(r.2.is_empty());
    }

    #[test]
    fn test_parse_response_hrule_between_sections() {
        let content = "Beschreibung\n\n---\n\n**Befehl:**\n```zsh\nls\n```\n\n**Parameter:**\n* foo bar";
        let result = parse_llm_response(content);
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(r.0.contains("Beschreibung"));
        assert!(!r.0.contains("---"));
    }

    #[test]
    fn test_parse_response_no_shell_block() {
        let content = "No shell command here, just Python code.\n\n```python\nprint(\"hello\")\n```";
        let result = parse_llm_response(&content);
        assert!(result.is_none());
    }

     #[test]
     fn test_parse_response_with_befehl_header() {
          // "Befehl:" header becomes "Befehl:" after stripping bold markers
         let content = "**Beschreibung:**\nTest\n\n**Befehl:**\n```bash\necho hello\n```";
         let result = parse_llm_response(content);
         assert!(result.is_some());
         let r = result.unwrap();
         assert!(r.0.contains("Beschreibung"));
         assert!(r.0.contains("Test"));
         assert_eq!(r.1, "echo hello");
         assert!(r.2.is_empty());
      }
}
