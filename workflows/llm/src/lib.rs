use lightflow::preload::*;

pub fn define() -> WorkflowSpec {
    workflow! {
        input "prompt": "text" {
            description: "Prompt text sent to the provider.",
            required: false,
            widget: "prompt",
        }
        input "text": "text" {
            description: "Alternate prompt input for composition with text-producing workflows.",
            required: false,
            widget: "textarea",
        }
        input "system": "text" {
            description: "Optional system prompt.",
            required: false,
            widget: "textarea",
        }
        input "preamble": "text" {
            description: "Optional Rig preamble text.",
            required: false,
            widget: "textarea",
        }
        input "provider": "text" {
            description: "Provider id such as openai-compatible, anthropic, ollama, openrouter, deepseek, xai, or mock.",
            required: false,
            default: "mock",
            choices: ["openai","openai-compatible","openai-responses","anthropic","ollama","openrouter","deepseek","xai","mock"],
            widget: "select",
        }
        input "model": "text" {
            description: "Provider model name.",
            required: true,
            widget: "text",
        }
        input "api_key": "text" {
            description: "Optional provider API key. Environment defaults are used when omitted.",
            required: false,
            widget: "password",
        }
        input "base_url": "text" {
            description: "Optional OpenAI-compatible or Ollama base URL.",
            required: false,
            widget: "url",
        }
        input "temperature": "number" {
            description: "Optional sampling temperature.",
            required: false,
            range: [0, 2, 0.1],
            widget: "slider",
        }
        input "max_tokens": "integer" {
            description: "Optional maximum output token count.",
            required: false,
            range: [1, 262144, 1],
            widget: "number",
        }
        input "additional_params": "json" {
            description: "Provider-specific JSON options passed through to the runtime.",
            required: false,
            default: {},
            widget: "json",
        }
        output "text": "text" {
            description: "Generated text.",
        }
        output "response": "text" {
            description: "Generated provider response text.",
        }
        output "provider": "text" {
            description: "Provider used for this call.",
        }
        output "model": "text" {
            description: "Model used for this call.",
        }
    }
        .name("RIG LLM")
        .description("Generate text through Rig using a provider, model, and runtime prompt.")
        .runtime("rig_runtime", "lightflow.llm.generate")
        .build()
}
