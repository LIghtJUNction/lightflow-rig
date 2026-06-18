# lightflow-rig

LightFlow workflow project for routing text generation through
[`rig-core`](https://crates.io/crates/rig-core).

## Workflow

- `lightflow.rig.llm`: prompt an arbitrary LLM provider through LightFlow's RIG runtime.

The workflow declares `lightflow.llm.generate`; the LightFlow dependency in
this project enables the `rig` feature so `lfw run` can execute it directly.

## Usage

```bash
lfw run lightflow.rig.llm \
  -i provider='"mock"' \
  -i model='"fake-llm"' \
  -i prompt='"hello"'
```

For real providers, set provider credentials in the environment or pass them
as inputs:

```bash
export OPENAI_API_KEY=...

lfw run lightflow.rig.llm \
  -i provider='"openai-compatible"' \
  -i model='"gpt-4o-mini"' \
  -i prompt='"Write a concise release note."' \
  -i temperature=0.2
```

Supported providers in the current runtime are `openai`,
`openai-compatible`, `openai-responses`, `anthropic`, `ollama`, `openrouter`,
`deepseek`, `xai`, and `mock`.
