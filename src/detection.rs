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
    let os = detect_os();
    let shell = detect_shell();
    format!(
         "You are a helpful assistant that converts natural language questions into shell commands.
The user's operating system is: {}.
The user's shell is: {}.

Structure the answer like this:

```bash
<command to execute>
```

Explaination:
<short explanation of the command>
<explaination of all used parameters>

---<if there are sensible Alternatives>
Alternatives

<display like above>",
        os, shell
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
    #[serial]
    fn test_build_system_prompt_contains_os_and_shell() {
        let prompt = build_system_prompt();
        let os = detect_os();
        let shell = detect_shell();
        assert!(prompt.contains(os), "Prompt should contain OS info: {}", prompt);
        assert!(prompt.contains(&shell), "Prompt should contain shell info: {}", prompt);
     }
}
