## Why

Developers frequently want to quickly translate natural language questions about shell commands into working commands. Currently, they must either remember command syntax or search documentation. A CLI tool that converts natural language queries into shell commands via LLM saves time and reduces friction in daily workflow.

## What Changes

- Create a Rust CLI binary named `plz` executable
- Accept natural language queries as positional arguments
- Send queries to an OpenAI-compatible LLM API with system prompt providing OS and shell context
- Display formatted output: description of the command, command with parameter explanation, and the raw command on a separate clearly marked line
- Load configuration from `~/.config/plz/plz.json` (endpoint, API key, model)
- If no model is configured, list available models from the API and show a hint message
- Use only the OpenAI API (chat completions endpoint)

## Capabilities

### New Capabilities
- `plz-cli`: Rust CLI tool that converts natural language shell command queries into formatted LLM-generated answers with model configuration via JSON config file and OpenAI API integration

### Modified Capabilities
- *(none)*

## Impact

- New Rust binary in workspace root as a separate crate alongside the flake/nix setup
- External dependency: OpenAI-compatible HTTP API (any endpoint)
- Configuration file at `~/.config/plz/plz.json`
- Requires Rust toolchain (already available)
