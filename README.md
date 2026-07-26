# lightflow-rig

LightFlow workflow project for routing text generation through
[`rig-core`](https://crates.io/crates/rig-core).

## Workflow

- `lightflow.rig_llm`: prompt an arbitrary LLM provider through LightFlow's RIG runtime.

The workflow declares `lightflow.llm.generate`; its published package owns the
Rig adapter and executable `runner.v1` boundary. The LightFlow host has no
provider implementation and requires no Rig feature.
The workflow includes Node Schema v1 metadata for editor palettes and an agent
skill under `.agent/skills/lightflow-rig-llm/SKILL.md`.

## Usage

```bash
lfw run lightflow.rig_llm \
  -i provider='"mock"' \
  -i model='"fake-llm"' \
  -i prompt='"hello"'
```

For real providers, credentials and endpoint configuration are environment
only. Undeclared inputs including `api_key` and `base_url` are rejected before
the package runner starts and are not persisted in run history:

```bash
export OPENAI_COMPATIBLE_BASE_URL='https://gateway.example/v1'
export OPENAI_COMPATIBLE_API_KEY='...'

lfw run lightflow.rig_llm \
  -i provider='"openai-compatible"' \
  -i model='"gpt-4o-mini"' \
  -i prompt='"Write a concise release note."' \
  -i temperature=0.2
```

Official OpenAI providers use `OPENAI_API_KEY` and no custom endpoint.
`openai-compatible` uses only the dedicated
`OPENAI_COMPATIBLE_BASE_URL` / `OPENAI_COMPATIBLE_API_KEY` pair and never
inherits `OPENAI_API_KEY`. The same contract applies through CLI, HTTP, and
MCP.

Supported providers in the current runtime are `openai`,
`openai-compatible`, `openai-responses`, `anthropic`, `ollama`, `openrouter`,
`deepseek`, `xai`, and `mock`.

## Node Conformance

Validate the workflow contract before publishing changes:

```bash
lfw node test lightflow.rig_llm
```
