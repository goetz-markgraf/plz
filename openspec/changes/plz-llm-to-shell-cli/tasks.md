## 1. Project Setup

- [x] 1.1 Create new Cargo binary crate `plz` within workspace with `clap`, `reqwest`, `tokio`, `serde`, `serde_json`, `dirs` dependencies
- [x] 1.2 Configure `Cargo.toml` with binary name `plz`, set edition to 2021, and add all dependency versions
- [x] 1.3 Update current `.gitignore` entry for `target/` and any generated artifacts if needed

## 2. Configuration Module

- [x] 2.1 Create `src/config.rs` with `Config` struct: `endpoint: String`, `api_key: String`, `model: Option<String>` (all fields `#[serde(default)]` for optional config)
- [x] 2.2 Implement `Config::load()` that reads `~/.config/plz/plz.json` using `dirs::config_dir()` or `~/` fallback
- [x] 2.3 Implement config validation: check that `endpoint` and `api_key` are present when executing a query
- [x] 2.4 Handle missing config file gracefully with clear error message and exit with code 1
- [x] 2.5 Handle invalid JSON with parse error details and exit with code 1

## 3. CLI Argument Parsing

- [x] 3.1 Create `src/cli.rs` with `clap` derive API for `PlzArgs` struct
- [x] 3.2 Define positional argument `query: String` for the natural language input
- [x] 3.3 Define `--model: Option<String>` CLI flag to override config model
- [x] 3.4 Define `--command-only: bool` flag for raw command output
- [x] 3.5 Define `-h, --help` and `-v, --version` flags using clap's built-in auto-derive
- [x] 3.6 Validate that query is not empty, show usage on empty, exit with code 1

## 4. OS and Shell Detection

- [x] 4.1 Create `src/detection.rs` with system info detection
- [x] 4.2 Implement `detect_os()` using `cfg!(target_os)` returning `macos`, `linux`, or `windows`
- [x] 4.3 Implement `detect_shell()` reading `$SHELL` env var, extracting shell name from path, defaulting to `unknown`
- [x] 4.4 Implement `build_system_prompt()` that returns a formatted string with OS and shell info

## 5. API Client

- [x] 5.1 Create `src/api.rs` with `PlzClient` struct holding endpoint and api_key
- [x] 5.2 Define OpenAI-compatible request/response structs (Message, ChatCompletionRequest, ChatCompletionResponse, etc.)
- [x] 5.3 Implement `chat_completion()` method that sends a POST to `{endpoint}/chat/completions`
- [x] 5.4 Set `Authorization: Bearer {api_key}` header and `Content-Type: application/json` header
- [x] 5.5 Map HTTP 401 to custom error with message about invalid API key
- [x] 5.6 Map HTTP 4xx/5xx to custom error with status code and message
- [x] 5.7 Handle network errors with descriptive message including the endpoint URL

## 6. Models Listing

- [x] 6.1 Create `src/models.rs` for listing available models
- [x] 6.2 Define OpenAI-compatible model response struct (ModelsResponse with data: Vec<Model>)
- [x] 6.3 Implement `list_models()` that calls `GET {endpoint}/models`
- [x] 6.4 Format and display the model list with id, name, and object type
- [x] 6.5 Add hint message prompting user to configure a model

## 7. Output Formatting

- [x] 7.1 Create `src/output.rs` with response formatting logic
- [x] 7.2 Implement `format_response()` that displays the LLM response with `> **Shell-Befehl:**` prefix wrapper
- [x] 7.3 Implement `extract_command()` that finds the first ` ```bash\n...\n``` ` block in response
- [x] 7.4 Implement `format_command_only()` that prints only the extracted command line
- [x] 7.5 Handle case where no code block exists: display full response with noting message

## 8. Main Application Logic

- [x] 8.1 Create `src/main.rs` as entry point
- [x] 8.2 Parse CLI args, load config, detect OS/shell
- [x] 8.3 If model is None, call model listing and exit after displaying results
- [x] 8.4 Build OpenAI request with system prompt (OS/shell info) and user query
- [x] 8.5 Call API, handle all error cases
- [x] 8.6 Format and display output based on `--command-only` flag

## 9. Build and Test

- [x] 9.1 Run `cargo build` and verify binary is produced at `target/debug/plz`
- [x] 9.2 Run `cargo test` to verify unit tests passes
- [ ] 9.3 Test with config missing: verify graceful error message
- [ ] 9.4 Test with invalid config JSON: verify parse error message
- [ ] 9.5 Test with valid config but no model: verify model listing works
- [ ] 9.6 Test with valid config and model: verify query execution works
- [ ] 9.7 Test `--command-only` flag: verify only command is output
- [ ] 9.8 Test `--model` flag: verify model override works
