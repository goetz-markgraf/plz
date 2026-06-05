# Capability: plz-cli

## Purpose

`plz` is a CLI tool that translates natural language queries into shell commands by calling an OpenAI-compatible LLM API. It detects the current OS and shell, sends the query with system context, and displays the response in a structured, human-readable format.

## Requirements

### Requirement: CLI query submission
The `plz` binary SHALL accept a natural language query as a positional argument and send it to an OpenAI-compatible API with a system prompt containing OS and shell information.

#### Scenario: Basic query submission
- **WHEN** the user runs `plz "wie kann ich die URL des origin des aktuellen git projektes feststellen?"`
- **THEN** the tool sends the query with system context (OS, shell) to the configured LLM endpoint and displays the formatted response

#### Scenario: Query with special characters
- **WHEN** the user runs `plz "show me all files with .rs extension"`
- **THEN** the tool properly JSON-encodes and sends the query to the API via HTTP POST body

#### Scenario: Empty query
- **WHEN** the user runs `plz` with no arguments
- **THEN** the tool outputs a usage message explaining the correct invocation and exits with a non-zero code

### Requirement: Command output format
The tool SHALL display the LLM response in a structured format with description, parameter explanation, and the raw command in a clearly delimited code block.

#### Scenario: Formatted response display
- **WHEN** the LLM returns a valid response containing a shell command
- **THEN** the tool displays: a brief description of the command (in cyan bold), parameter explanations (if present), and the command highlighted with a `Shell-Befehl:` label (cyan bold) followed by the command itself rendered with a green background — all via ANSI terminal styling

#### Scenario: Response without code block
- **WHEN** the LLM returns text without a code block
- **THEN** the tool displays the response as-is, noting that no executable command was found

### Requirement: Configuration file loading
The tool SHALL load configuration from `~/.config/plz/plz.json` supporting `endpoint`, `api_key`, and `model` fields. All fields are required for query execution except `model` (which is optional if model listing is desired).

#### Scenario: Valid config file exists
- **WHEN** `~/.config/plz/plz.json` exists with valid JSON containing `endpoint`, `api_key`, and `model`
- **THEN** the tool loads the configuration and uses it for API calls

#### Scenario: Config file missing
- **WHEN** `~/.config/plz/plz.json` does not exist
- **THEN** the tool outputs an error message with the expected config path and exits with a non-zero code

#### Scenario: Invalid config format
- **WHEN** `~/.config/plz/plz.json` contains invalid JSON
- **THEN** the tool outputs a clear error message indicating the JSON parsing failure and exits with a non-zero code

### Requirement: Model listing
When `model` is not configured, the tool SHALL call the API's `/models` endpoint and display the available models as an alternative to executing a query.

#### Scenario: No model configured
- **WHEN** the config file exists but `model` field is omitted
- **THEN** the tool displays a hint message and lists available models from the API with a message prompting the user to configure a model

#### Scenario: Model listing with no API access
- **WHEN** the model listing call fails due to network or auth error
- **THEN** the tool displays the error message and suggests configuring a model with `--model` flag

### Requirement: CLI help and version flags
The tool SHALL support `-h, --help` for usage information and `-v, --version` for version display via clap's built-in auto-derive.

#### Scenario: Help flag displays usage
- **WHEN** the user runs `plz -h` or `plz --help`
- **THEN** the tool displays the usage information including all flags and positional arguments and exits gracefully

#### Scenario: Version flag displays version
- **WHEN** the user runs `plz -v` or `plz --version`
- **THEN** the tool displays the binary version and exits gracefully

### Requirement: CLI model override
The tool SHALL support a `--model` flag that overrides the model from the config file at runtime.

#### Scenario: Model flag overrides config
- **WHEN** the user runs `plz --model gpt-4 "list all running processes"`
- **THEN** the tool uses `gpt-4` instead of the model specified in the config file

### Requirement: Raw command output
The tool SHALL support a `--command-only` flag that outputs only the command without description or parameter explanation.

#### Scenario: Command-only output
- **WHEN** the user runs `plz --command-only "list all files in current directory"`
- **THEN** the tool outputs only the shell command line (no markdown, no description)

### Requirement: OS and shell detection
The tool SHALL automatically detect the current operating system (macOS, Linux, Windows) and shell (bash, zsh, fish, etc.) and inject this information into the system prompt.

#### Scenario: Correct OS detection
- **WHEN** the user runs `plz` on macOS
- **THEN** the system prompt includes the correct OS information

#### Scenario: Shell detection from environment
- **WHEN** the user runs `plz` with `$SHELL` set to `/bin/zsh`
- **THEN** the system prompt includes `zsh` as the target shell

### Requirement: Error handling
The tool SHALL handle and display clear error messages for all failure modes: invalid config, API errors, network issues, and invalid responses.

#### Scenario: Invalid API key
- **WHEN** the API returns a 401 Unauthorized
- **THEN** the tool outputs a clear message: "Invalid API key. Please check your configuration at ~/.config/plz/plz.json"

#### Scenario: Network error
- **WHEN** the API endpoint is unreachable
- **THEN** the tool outputs a message indicating network error with the endpoint URL and suggests checking connectivity

#### Scenario: API returns error
- **WHEN** the API returns an error status code (4xx or 5xx) other than 401
- **THEN** the tool outputs the error status and message from the API response
