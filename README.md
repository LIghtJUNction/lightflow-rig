# lightflow-rig

LightFlow workflow project for routing text generation through
[`rig-core`](https://crates.io/crates/rig-core).

## Workflow

- `lightflow.rig_llm`: prompt an arbitrary LLM provider through LightFlow's RIG runtime.

The workflow declares `lightflow.llm.generate`; the LightFlow dependency in
this project enables the `rig` feature so `lfw run` can execute it directly.
The workflow includes Node Schema v1 metadata for editor palettes and an agent
skill under `.agent/skills/lightflow-rig-llm/SKILL.md`.

## Usage

```bash
lfw run lightflow.rig_llm \
  -i provider='"mock"' \
  -i model='"fake-llm"' \
  -i prompt='"hello"'
```

For real providers, set provider credentials in the environment or pass them
as inputs:

```bash
export OPENAI_API_KEY=...

lfw run lightflow.rig_llm \
  -i provider='"openai-compatible"' \
  -i model='"gpt-4o-mini"' \
  -i prompt='"Write a concise release note."' \
  -i temperature=0.2
```

Supported providers in the current runtime are `openai`,
`openai-compatible`, `openai-responses`, `anthropic`, `ollama`, `openrouter`,
`deepseek`, `xai`, and `mock`.

## Node Conformance

Validate the workflow contract before publishing changes:

```bash
lfw node test lightflow.rig_llm
```
