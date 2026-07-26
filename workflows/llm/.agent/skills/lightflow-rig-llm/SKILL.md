---
name: LightFlow RIG LLM
description: Use this skill when working with the lightflow.rig_llm workflow, routing prompts through Rig to OpenAI-compatible APIs, Anthropic, Ollama, OpenRouter, DeepSeek, xAI, or local test providers.
version: 0.1.0
---

# LightFlow RIG LLM

Use `lightflow.rig_llm` to call an arbitrary LLM through LightFlow's RIG-backed runtime.

## Workflow

- Workflow id: `lightflow.rig_llm`
- Required inputs: `prompt` or `text`, and `model`.
- Common inputs: `provider`, `system`, `preamble`, `temperature`, `max_tokens`, `additional_params`.
- Outputs: `text`, `response`, `provider`, `model`.
- Runtime capability: `lightflow.llm.generate`.
- Runtime engine: `runner.v1`.
- Node Schema: `provider` is a select control; `temperature` and `max_tokens` expose editor ranges; `additional_params` is a JSON editor.

## Runtime

The workflow crate owns the Rig provider adapter and exposes it through its
package runner. The LightFlow host does not need a Rig feature or provider
implementation. For local contract checks, use `provider=mock` and any model
name. Keep provider API keys only in runtime environment variables; secrets
are not accepted as workflow inputs.

## Providers

Set `provider` to one of:

- `openai` or `openai-compatible`
- `openai-responses`
- `anthropic`
- `ollama`
- `openrouter`
- `deepseek`
- `xai`
- `mock` for local tests

Provider endpoints and secrets are administrator-owned environment
configuration and are not accepted as workflow inputs. Provider secrets are
read only from variables such as `OPENAI_API_KEY`,
`ANTHROPIC_API_KEY`, `OLLAMA_API_KEY`, `OPENROUTER_API_KEY`,
`DEEPSEEK_API_KEY`, and `XAI_API_KEY`. OpenAI-compatible endpoints use the
dedicated pair `OPENAI_COMPATIBLE_BASE_URL` and
`OPENAI_COMPATIBLE_API_KEY`; they never receive `OPENAI_API_KEY`. Ollama may
use `OLLAMA_API_BASE_URL`. `LIGHTFLOW_RIG_PROVIDER`,
`LIGHTFLOW_RIG_MODEL`, and `LIGHTFLOW_RIG_SYSTEM` provide defaults.

## Examples

Local mock verification:

```bash
lfw run lightflow.rig_llm \
  -i provider='"mock"' \
  -i model='"fake-llm"' \
  -i prompt='"hello"'
```

OpenAI-compatible endpoint:

```bash
export OPENAI_COMPATIBLE_BASE_URL='https://gateway.example/v1'
export OPENAI_COMPATIBLE_API_KEY='...'
lfw run lightflow.rig_llm \
  -i provider='"openai-compatible"' \
  -i model='"gpt-4o-mini"' \
  -i prompt='"Write a concise status update."'
```

Ollama:

```bash
lfw run lightflow.rig_llm \
  -i provider='"ollama"' \
  -i model='"qwen2.5:14b"' \
  -i prompt='"Summarize this workflow project."'
```

## Validation

```bash
lfw node test lightflow.rig_llm
```
## API Usage

Start `lfw serve`, then call the workflow through the shared HTTP run contract. Adjust `inputs` to match the workflow contract above.

```bash
curl -sS -X POST http://127.0.0.1:5174/workflows/lightflow.rig_llm/run \
  -H 'content-type: application/json' \
  -d '{"inputs":{}}'
```
