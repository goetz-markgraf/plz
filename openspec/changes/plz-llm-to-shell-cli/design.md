## Context

The workspace already uses Rust (rustc 1.93.1, cargo 1.93.1) with a Nix flake setup. The tool is a standalone CLI binary with no existing code to integrate with. External API dependency: any OpenAI-compatible chat completions endpoint.

## Goals / Non-Goals

**Goals:**
- Rust CLI binary (`plz`) that sends queries to an OpenAI-compatible API
- Clean configuration via `~/.config/plz/plz.json`
- Formatted output: description, parameter explanation, raw command
- Auto-detection of OS and shell info to prepend to the LLM prompt
- Model listing when no model is configured

**Non-Goals:**
- Authentication or account management
- Caching or history of previous queries
- Subcommands (everything is a single query)
- Streaming output
- Support for non-OpenAI-compatible APIs beyond basic chat completions

## Decisions

### 1. Single binary, `clap` for CLI parsing
Use `clap` with derive API for argparse. It's the standard, well-documented, and produces a clean CLI with auto-generated help.

### 2. `reqwest` for HTTP requests, `serde`/`serde_json` for JSON
`reqwest` with the `json` feature handles HTTP calls and JSON serialization. `tokio` as async runtime.

### 3. Config format: JSON in `XDG_CONFIG_HOME` or `~/.config/plz/plz.json`
Simplest approach. Uses `dirs` crate to find user config dir, with fallback to `~/.config/plz/plz.json`.

Config structure:
```json
{
  "endpoint": "https://api.openai.com/v1",
  "api_key": "sk-...",
  "model": "gpt-4o-mini"
}
```
All three fields optional if model listing is desired (model can be omitted).

**Choice rationale**: JSON is universally understood. YAML would require an extra dependency.

### 4. System info injection
Detect OS via `cfg!(target_os)` and shell via `$SHELL` or `SHLVL` env vars. Inject as a system prompt so the LLM targets the correct environment.

### 5. Model listing
When `model` is absent in config, call `GET /models` from the configured endpoint and display the list. The user can then configure a model.

### 6. Output format

Terminal output uses ANSI escape codes (no raw markdown printed to the terminal):

```
<description>        ← cyan bold
Parameter:           ← bold (only shown when params are present)
   - <param line>
Shell-Befehl:        ← cyan bold label
<command>            ← green background highlight
```

If the LLM response contains no recognisable shell code block, the raw response text is printed followed by a yellow note: `(Note: No executable command found in the response.)`

## Risks / Trade-offs

- **[Risk] API costs** → Mitigation: User provides their own API key, no costs for tool author. Document that different models have different costs.
- **[Risk] API downtime** → Mitigation: Propagate clear error messages from the API.
- **[Risk] Long LLM responses** → Mitigation: Render full response but ensure the command block is clearly delimited and easy to copy.

## Open Questions

- Should the tool support a `--model` CLI flag to override the config model at runtime? (Recommended: yes, for flexibility)
- Should there be a `--raw` or `--command-only` flag to output just the command without extra text? (Recommended: yes, for scripting use cases)
