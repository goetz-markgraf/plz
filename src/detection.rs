pub fn detect_os() -> &'static str {
    cfg!(target_os = "macos").then_some("macos")
        .or(cfg!(target_os = "linux").then_some("linux"))
        .or(cfg!(target_os = "windows").then_some("windows"))
        .unwrap_or("unknown")
}

pub fn detect_shell() -> String {
    std::env::var("SHELL")
        .ok()
        .and_then(|shell| shell.rsplit('/').next().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn build_system_prompt() -> String {
    "You are a helpful Assistant. Your answers are always short and to the point. Always answer in Json Format. The target format will be part of the user request.".to_string()
}

pub fn build_user_prompt(query: &str) -> String {
    let os = detect_os();
    let shell = detect_shell();
    format!(
        r#"Create a shell script that fulfills this demand: {query}
It must always fit the users operating system and used shell:

User OS: {os}
User Shell: {shell}

Give the answer as a Json formatted like this

{{
   "command": "<only the command, maybe with place holders for parameters' values>",
   "desc": "<short description (2-3 sentences) what the code does>",
   "params": [
     {{
        "param": "<parameter like it is in the 'command'>",
        "desc": "<describe shortly what this param does>"
     }}
   ],
   "alt": [
     {{
       "command": "<only the command, maybe with place holders for parameters' values>",
       "desc": "<short description (2-3 sentences) what the code does>",
       "params": [
         {{
            "param": "<parameter like it is in the 'command'>",
            "desc": "<describe shortly what this param does>"
         }}
       ]
     }}
   ]
}}

Rules:
- Every parameter, flag, or placeholder that appears in the 'command' string MUST have an entry in 'params'.
- Do not leave 'params' empty if the command contains flags or placeholders.
- Only fill the 'alt' field if there are meaningful alternatives."#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_detect_os_returns_known_value() {
        let os = detect_os();
        assert!(
             ["macos", "linux", "windows", "unknown"].contains(&os),
             "OS should be one of known values, got: {}",
            os
         );
     }

    #[test]
    fn test_detect_os_non_empty() {
        assert!(!detect_os().is_empty());
    }

    #[test]
    #[serial]
    fn test_detect_shell_without_env_var() {
         // Temporarily unset SHELL
        let had_shell = std::env::var("SHELL").ok();
        std::env::remove_var("SHELL");
        assert_eq!(detect_shell(), "unknown");
        if let Some(shell) = had_shell {
            std::env::set_var("SHELL", shell);
         }
     }

    #[test]
    #[serial]
    fn test_detect_shell_with_path() {
        std::env::set_var("SHELL", "/bin/zsh");
        assert_eq!(detect_shell(), "zsh");

        std::env::set_var("SHELL", "/usr/local/bin/fish");
        assert_eq!(detect_shell(), "fish");

        if let Some(shell) = std::env::var("SHELL").ok() {
            std::env::set_var("SHELL", shell);
         }
     }

    #[test]
    fn test_build_system_prompt_is_static() {
        let prompt = build_system_prompt();
        assert!(prompt.contains("Json Format"), "System prompt should mention Json Format: {}", prompt);
    }

    #[test]
    #[serial]
    fn test_build_user_prompt_contains_os_and_shell() {
        let prompt = build_user_prompt("list files");
        let os = detect_os();
        let shell = detect_shell();
        assert!(prompt.contains(os), "User prompt should contain OS info: {}", prompt);
        assert!(prompt.contains(&shell), "User prompt should contain shell info: {}", prompt);
        assert!(prompt.contains("list files"), "User prompt should contain the query: {}", prompt);
    }
}
