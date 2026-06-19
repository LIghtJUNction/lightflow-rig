---
name: LightFlow RIG LLM
description: Use this skill when working with the lightflow.rig.llm workflow, routing prompts through Rig to OpenAI-compatible APIs, Anthropic, Ollama, OpenRouter, DeepSeek, xAI, or local test providers.
version: 0.1.0
---

# LightFlow RIG LLM

Use `lightflow.rig.llm` to call an arbitrary LLM through LightFlow's RIG-backed runtime.

## Workflow

- Workflow id: `lightflow.rig.llm`
- Required inputs: `prompt` or `text`, and `model`.
- Common inputs: `provider`, `system`, `preamble`, `api_key`, `base_url`, `temperature`, `max_tokens`, `additional_params`.
- Outputs: `text`, `response`, `provider`, `model`.
- Runtime capability: `lightflow.llm.generate`.
- Node Schema: `provider` is a select control; `temperature` and `max_tokens` expose editor ranges; `additional_params` is a JSON editor.

## Runtime

Run with a LightFlow binary compiled with `--features rig`. For local contract checks, use `provider=mock` and any model name. For real providers, prefer environment variables for secrets and pass `api_key` only for temporary local runs.

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

`api_key` and `base_url` can be passed as workflow inputs. If omitted, LightFlow reads provider-specific environment variables such as `OPENAI_API_KEY`, `OPENAI_BASE_URL`, `ANTHROPIC_API_KEY`, `OLLAMA_API_BASE_URL`, `OPENROUTER_API_KEY`, `DEEPSEEK_API_KEY`, and `XAI_API_KEY`. `LIGHTFLOW_RIG_PROVIDER`, `LIGHTFLOW_RIG_MODEL`, and `LIGHTFLOW_RIG_SYSTEM` provide defaults.

## Examples

Local mock verification:

```bash
lfw run lightflow.rig.llm \
  -i provider='"mock"' \
  -i model='"fake-llm"' \
  -i prompt='"hello"'
```

OpenAI-compatible endpoint:

```bash
lfw run lightflow.rig.llm \
  -i provider='"openai-compatible"' \
  -i model='"gpt-4o-mini"' \
  -i prompt='"Write a concise status update."' \
  -i base_url='"https://api.openai.com/v1"'
```

Ollama:

```bash
lfw run lightflow.rig.llm \
  -i provider='"ollama"' \
  -i model='"qwen2.5:14b"' \
  -i prompt='"Summarize this workflow project."'
```

## Validation

```bash
lfw node test lightflow.rig.llm
```
