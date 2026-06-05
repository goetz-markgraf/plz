/// Integration tests for the `plz` binary (tasks 9.3 – 9.8).
///
/// Each test launches the compiled binary as a child process, controls the
/// `HOME` environment variable so config resolution is fully isolated, and
/// asserts on exit code + stderr/stdout content.
///
/// A mock HTTP server (via `mockito`) is used wherever the binary makes
/// real network calls (model listing, query execution).
use std::fs;
use std::path::PathBuf;
use std::process::Command;

// ── helpers ──────────────────────────────────────────────────────────────────

/// Absolute path to the compiled `plz` binary.
fn bin() -> PathBuf {
    let mut p = std::env::current_exe()
        .expect("could not locate test binary")
        .parent()
        .unwrap()
        .to_path_buf();
    // `current_exe` is somewhere inside `target/debug/deps/`; walk up to `target/debug/`
    if p.ends_with("deps") {
        p = p.parent().unwrap().to_path_buf();
    }
    p.join("plz")
}

/// Create an isolated temp home directory and write a config file there.
/// Returns (temp_dir, config_path) — keep `temp_dir` alive for the test
/// duration so the directory is not cleaned up too early.
fn temp_home_with_config(content: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let config_dir = dir.path().join(".config").join("plz");
    fs::create_dir_all(&config_dir).expect("create config dir");
    let config_path = config_dir.join("plz.json");
    fs::write(&config_path, content).expect("write config");
    (dir, config_path)
}

/// Create a temp home directory with **no** config file.
fn temp_home_no_config() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

// ── 9.3: config file missing ──────────────────────────────────────────────────

/// Task 9.3 — running `plz` with no config file produces a clear error and
/// exits with code 1.
#[test]
fn test_config_missing_exits_with_error() {
    let home = temp_home_no_config();

    let output = Command::new(bin())
        .arg("list files")
        .env("HOME", home.path())
        // Unset XDG_CONFIG_HOME so dirs crate doesn't find a stale config dir
        .env_remove("XDG_CONFIG_HOME")
        .output()
        .expect("failed to run plz");

    assert!(
        !output.status.success(),
        "expected non-zero exit code when config is missing"
    );
    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found") || stderr.contains("Configuration file"),
        "expected config-not-found message in stderr, got: {stderr}"
    );
}

// ── 9.4: invalid config JSON ──────────────────────────────────────────────────

/// Task 9.4 — running `plz` with a malformed JSON config produces a parse
/// error and exits with code 1.
#[test]
fn test_invalid_config_json_exits_with_error() {
    let (home, _cfg) = temp_home_with_config("{ this is not valid json }");

    let output = Command::new(bin())
        .arg("list files")
        .env("HOME", home.path())
        .env_remove("XDG_CONFIG_HOME")
        .output()
        .expect("failed to run plz");

    assert!(
        !output.status.success(),
        "expected non-zero exit code for invalid JSON config"
    );
    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("parse") || stderr.contains("JSON") || stderr.contains("Failed"),
        "expected JSON parse error in stderr, got: {stderr}"
    );
}

// ── 9.5: model listing when no model is configured ────────────────────────────

/// Task 9.5 — when `model` is absent from the config, `plz` calls
/// `GET /models` and displays the list, then exits with code 0.
#[test]
fn test_no_model_lists_models() {
    // Spin up a mock HTTP server
    let mut server = mockito::Server::new();
    let _mock = server
        .mock("GET", "/models")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{"data":[{"id":"gpt-4o","name":"GPT-4o","object":"model"},{"id":"gpt-3.5-turbo","name":"GPT-3.5 Turbo","object":"model"}]}"#,
        )
        .create();

    let config = format!(
        r#"{{"endpoint":"{}","api_key":"test-key"}}"#,
        server.url()
    );
    let (home, _cfg) = temp_home_with_config(&config);

    let output = Command::new(bin())
        .arg("list running processes")
        .env("HOME", home.path())
        .env_remove("XDG_CONFIG_HOME")
        .output()
        .expect("failed to run plz");

    assert!(
        output.status.success(),
        "expected exit 0 when listing models, got: {:?}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("gpt-4o") || stdout.contains("Available"),
        "expected model list in stdout, got: {stdout}"
    );
    assert!(
        stdout.contains("configure") || stdout.contains("model"),
        "expected hint about configuring a model, got: {stdout}"
    );
}

// ── 9.6: full query execution ─────────────────────────────────────────────────

/// Task 9.6 — with a valid config and model, `plz` sends the query to the API
/// and displays the formatted response.
#[test]
fn test_query_execution_with_valid_config() {
    let mut server = mockito::Server::new();
    let _mock = server
        .mock("POST", "/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{"choices":[{"message":{"role":"assistant","content":"List all files:\n\n```bash\nls -la\n```\n\nExplanation: lists directory contents."}}]}"#,
        )
        .create();

    let config = format!(
        r#"{{"endpoint":"{}","api_key":"test-key","model":"gpt-4o"}}"#,
        server.url()
    );
    let (home, _cfg) = temp_home_with_config(&config);

    let output = Command::new(bin())
        .arg("list all files in current directory")
        .env("HOME", home.path())
        .env_remove("XDG_CONFIG_HOME")
        .output()
        .expect("failed to run plz");

    assert!(
        output.status.success(),
        "expected exit 0 for successful query, got: {:?}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("ls -la") || stdout.contains("Shell-Befehl"),
        "expected command in stdout, got: {stdout}"
    );
}

// ── 9.7: --command-only flag ──────────────────────────────────────────────────

/// Task 9.7 — `--command-only` outputs only the raw command, no description
/// or markdown.
#[test]
fn test_command_only_flag() {
    let mut server = mockito::Server::new();
    let _mock = server
        .mock("POST", "/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{"choices":[{"message":{"role":"assistant","content":"Here is the command:\n\n```bash\nls -la\n```\n\nThis lists files."}}]}"#,
        )
        .create();

    let config = format!(
        r#"{{"endpoint":"{}","api_key":"test-key","model":"gpt-4o"}}"#,
        server.url()
    );
    let (home, _cfg) = temp_home_with_config(&config);

    let output = Command::new(bin())
        .args(["--command-only", "list all files in current directory"])
        .env("HOME", home.path())
        .env_remove("XDG_CONFIG_HOME")
        .output()
        .expect("failed to run plz");

    assert!(
        output.status.success(),
        "expected exit 0 for --command-only, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Only the bare command should appear; no description noise
    assert!(
        stdout.trim() == "ls -la",
        "expected only 'ls -la' in stdout, got: {stdout:?}"
    );
}

// ── 9.8: --model flag overrides config ────────────────────────────────────────

/// Task 9.8 — `--model <name>` overrides the model from the config file.
/// We verify by inspecting the request body sent to the mock server.
#[test]
fn test_model_flag_overrides_config() {
    let mut server = mockito::Server::new();
    // The mock matches only if the request body contains the overridden model id
    let _mock = server
        .mock("POST", "/chat/completions")
        .match_body(mockito::Matcher::PartialJsonString(
            r#"{"model":"my-special-model"}"#.to_string(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{"choices":[{"message":{"role":"assistant","content":"```bash\necho ok\n```"}}]}"#,
        )
        .create();

    // Config has a different model; --model flag should win
    let config = format!(
        r#"{{"endpoint":"{}","api_key":"test-key","model":"config-model"}}"#,
        server.url()
    );
    let (home, _cfg) = temp_home_with_config(&config);

    let output = Command::new(bin())
        .args(["--model", "my-special-model", "say hello"])
        .env("HOME", home.path())
        .env_remove("XDG_CONFIG_HOME")
        .output()
        .expect("failed to run plz");

    assert!(
        output.status.success(),
        "expected exit 0 when --model overrides config, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // The mock would only match if "my-special-model" was used in the request;
    // if not, mockito returns 501 which would cause a non-zero exit.
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("echo ok"),
        "expected command from mocked response, got: {stdout}"
    );
}
