use lightflow::preload::*;
use lightflow::runner::Response;
use lightflow::serde_json::{Map, Value, json};
use std::error::Error;
use std::fmt;

pub const WORKFLOW_ID: &str = "lightflow.rig_llm";
pub const WORKFLOW_VERSION: &str = env!("CARGO_PKG_VERSION");
const SOURCE_DIGEST: u64 = source_digest(include_bytes!("lib.rs"));

pub fn define() -> WorkflowSpec {
    workflow! {
        name: "RIG LLM",
        description: "Generate text through Rig using a provider, model, and runtime prompt.",
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
        .builtin_runtime("rig_runtime", "lightflow.llm.generate", "runner.v1")
        .build()
}

/// Execute the package-owned Rig provider adapter.
pub fn execute(inputs: &Map<String, Value>) -> Result<Response, RigError> {
    let request = RigRequest::from_inputs(inputs)?;
    let response = if request.provider == "mock" {
        format!("mock:{}:{}", request.model, request.prompt)
    } else {
        block_on(run_prompt(&request))??
    };

    Ok(Response {
        outputs: Map::from_iter([
            ("text".to_owned(), response.clone().into()),
            ("response".to_owned(), response.into()),
            ("provider".to_owned(), request.provider.into()),
            ("model".to_owned(), request.model.into()),
        ]),
        artifacts: Vec::new(),
        replay_fingerprint: Map::from_iter([
            ("crate_version".to_owned(), json!(WORKFLOW_VERSION)),
            (
                "implementation".to_owned(),
                json!(implementation_identity()),
            ),
        ]),
    })
}

/// Return deterministic identity for the adapter source used by replay checks.
pub fn implementation_identity() -> String {
    format!("lightflow.rig_llm.source.fnv1a64:{SOURCE_DIGEST:016x}")
}

#[derive(Debug, Clone)]
struct RigRequest {
    provider: String,
    model: String,
    prompt: String,
    system: Option<String>,
    api_key: Option<String>,
    base_url: Option<String>,
    temperature: Option<f64>,
    max_tokens: Option<u64>,
    additional_params: Option<Value>,
}

impl RigRequest {
    fn from_inputs(inputs: &Map<String, Value>) -> Result<Self, RigError> {
        const ALLOWED_INPUTS: &[&str] = &[
            "prompt",
            "text",
            "system",
            "preamble",
            "provider",
            "model",
            "temperature",
            "max_tokens",
            "additional_params",
        ];
        if let Some(name) = inputs
            .keys()
            .find(|name| !ALLOWED_INPUTS.contains(&name.as_str()))
        {
            return Err(RigError::UnknownInput(name.clone()));
        }
        let provider = optional_string(inputs, "provider")?
            .map(str::to_owned)
            .or_else(|| non_empty_env("LIGHTFLOW_RIG_PROVIDER"))
            .unwrap_or_else(|| "mock".to_owned())
            .to_ascii_lowercase();
        let model = optional_string(inputs, "model")?
            .map(str::to_owned)
            .or_else(|| non_empty_env("LIGHTFLOW_RIG_MODEL"))
            .ok_or(RigError::MissingInput("model"))?;
        let prompt = optional_string(inputs, "prompt")?
            .or(optional_string(inputs, "text")?)
            .map(str::to_owned)
            .ok_or(RigError::MissingInput("prompt or text"))?;
        let system = optional_string(inputs, "system")?
            .or(optional_string(inputs, "preamble")?)
            .map(str::to_owned)
            .or_else(|| non_empty_env("LIGHTFLOW_RIG_SYSTEM"));
        let api_key = env_api_key(&provider);
        let base_url = env_base_url(&provider);
        if matches!(
            provider.as_str(),
            "openai-compatible" | "openai_chat" | "openai-chat"
        ) && base_url.is_none()
        {
            return Err(RigError::MissingEndpoint(
                "OPENAI_COMPATIBLE_BASE_URL",
                provider,
            ));
        }
        let temperature = optional_number(inputs, "temperature")?;
        let max_tokens = optional_u64(inputs, "max_tokens")?;
        let additional_params = optional_object(inputs, "additional_params")?;

        Ok(Self {
            provider,
            model,
            prompt,
            system,
            api_key,
            base_url,
            temperature,
            max_tokens,
            additional_params,
        })
    }
}

async fn run_prompt(request: &RigRequest) -> Result<String, RigError> {
    match request.provider.as_str() {
        "openai" | "openai-compatible" | "openai_chat" | "openai-chat" => {
            let key_env = if request.provider == "openai" {
                "OPENAI_API_KEY"
            } else {
                "OPENAI_COMPATIBLE_API_KEY"
            };
            let api_key = required_api_key(request, key_env)?;
            let mut builder =
                rig_core::providers::openai::CompletionsClient::builder().api_key(&api_key);
            if let Some(base_url) = &request.base_url {
                builder = builder.base_url(base_url);
            }
            prompt_with_client(builder.build().map_err(rig_error)?, request).await
        }
        "openai-responses" | "openai_responses" => {
            let api_key = required_api_key(request, "OPENAI_API_KEY")?;
            let mut builder = rig_core::providers::openai::Client::builder().api_key(&api_key);
            if let Some(base_url) = &request.base_url {
                builder = builder.base_url(base_url);
            }
            prompt_with_client(builder.build().map_err(rig_error)?, request).await
        }
        "anthropic" | "claude" => {
            let api_key = required_api_key(request, "ANTHROPIC_API_KEY")?;
            let mut builder = rig_core::providers::anthropic::Client::builder().api_key(&api_key);
            if let Some(base_url) = &request.base_url {
                builder = builder.base_url(base_url);
            }
            prompt_with_client(builder.build().map_err(rig_error)?, request).await
        }
        "ollama" => {
            let mut builder = rig_core::providers::ollama::Client::builder()
                .api_key(request.api_key.clone().unwrap_or_default());
            if let Some(base_url) = &request.base_url {
                builder = builder.base_url(base_url);
            }
            prompt_with_client(builder.build().map_err(rig_error)?, request).await
        }
        "openrouter" => {
            let api_key = required_api_key(request, "OPENROUTER_API_KEY")?;
            let mut builder = rig_core::providers::openrouter::Client::builder().api_key(&api_key);
            if let Some(base_url) = &request.base_url {
                builder = builder.base_url(base_url);
            }
            prompt_with_client(builder.build().map_err(rig_error)?, request).await
        }
        "deepseek" => {
            let api_key = required_api_key(request, "DEEPSEEK_API_KEY")?;
            let mut builder = rig_core::providers::deepseek::Client::builder().api_key(&api_key);
            if let Some(base_url) = &request.base_url {
                builder = builder.base_url(base_url);
            }
            prompt_with_client(builder.build().map_err(rig_error)?, request).await
        }
        "xai" | "x.ai" => {
            let api_key = required_api_key(request, "XAI_API_KEY")?;
            let mut builder = rig_core::providers::xai::Client::builder().api_key(&api_key);
            if let Some(base_url) = &request.base_url {
                builder = builder.base_url(base_url);
            }
            prompt_with_client(builder.build().map_err(rig_error)?, request).await
        }
        provider => Err(RigError::UnsupportedProvider(provider.to_owned())),
    }
}

async fn prompt_with_client<C>(client: C, request: &RigRequest) -> Result<String, RigError>
where
    C: rig_core::client::completion::CompletionClient,
    C::CompletionModel: 'static,
{
    use rig_core::completion::Prompt;

    let mut builder = client.agent(request.model.clone());
    if let Some(system) = &request.system {
        builder = builder.preamble(system);
    }
    if let Some(temperature) = request.temperature {
        builder = builder.temperature(temperature);
    }
    if let Some(max_tokens) = request.max_tokens {
        builder = builder.max_tokens(max_tokens);
    }
    if let Some(additional_params) = &request.additional_params {
        builder = builder.additional_params(additional_params.clone());
    }
    builder
        .build()
        .prompt(request.prompt.clone())
        .await
        .map_err(rig_error)
}

fn block_on<F>(future: F) -> Result<F::Output, RigError>
where
    F: std::future::Future,
{
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        Ok(tokio::task::block_in_place(|| handle.block_on(future)))
    } else {
        Ok(tokio::runtime::Runtime::new()
            .map_err(|error| RigError::Provider(error.to_string()))?
            .block_on(future))
    }
}

fn required_api_key(request: &RigRequest, env_key: &'static str) -> Result<String, RigError> {
    request
        .api_key
        .clone()
        .ok_or(RigError::MissingApiKey(env_key, request.provider.clone()))
}

fn env_api_key(provider: &str) -> Option<String> {
    non_empty_env(match provider {
        "openai-compatible" | "openai_chat" | "openai-chat" => "OPENAI_COMPATIBLE_API_KEY",
        "anthropic" | "claude" => "ANTHROPIC_API_KEY",
        "ollama" => "OLLAMA_API_KEY",
        "openrouter" => "OPENROUTER_API_KEY",
        "deepseek" => "DEEPSEEK_API_KEY",
        "xai" | "x.ai" => "XAI_API_KEY",
        _ => "OPENAI_API_KEY",
    })
}

fn env_base_url(provider: &str) -> Option<String> {
    match provider {
        "openai-compatible" | "openai_chat" | "openai-chat" => {
            non_empty_env("OPENAI_COMPATIBLE_BASE_URL")
        }
        "ollama" => non_empty_env("OLLAMA_API_BASE_URL"),
        _ => None,
    }
}

fn non_empty_env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|value| !value.is_empty())
}

fn optional_string<'a>(
    inputs: &'a Map<String, Value>,
    name: &'static str,
) -> Result<Option<&'a str>, RigError> {
    inputs
        .get(name)
        .map(|value| value.as_str().ok_or(RigError::InvalidInputType(name)))
        .transpose()
}

fn optional_number(
    inputs: &Map<String, Value>,
    name: &'static str,
) -> Result<Option<f64>, RigError> {
    inputs
        .get(name)
        .map(|value| value.as_f64().ok_or(RigError::InvalidInputType(name)))
        .transpose()
}

fn optional_u64(inputs: &Map<String, Value>, name: &'static str) -> Result<Option<u64>, RigError> {
    inputs
        .get(name)
        .map(|value| value.as_u64().ok_or(RigError::InvalidInputType(name)))
        .transpose()
}

fn optional_object(
    inputs: &Map<String, Value>,
    name: &'static str,
) -> Result<Option<Value>, RigError> {
    inputs
        .get(name)
        .map(|value| {
            value
                .as_object()
                .map(|_| value.clone())
                .ok_or(RigError::InvalidInputType(name))
        })
        .transpose()
}

fn rig_error(error: impl fmt::Display) -> RigError {
    RigError::Provider(error.to_string())
}

#[derive(Debug)]
pub enum RigError {
    MissingInput(&'static str),
    UnknownInput(String),
    InvalidInputType(&'static str),
    MissingApiKey(&'static str, String),
    MissingEndpoint(&'static str, String),
    UnsupportedProvider(String),
    Provider(String),
}

impl fmt::Display for RigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingInput(name) => write!(formatter, "required input `{name}` is missing"),
            Self::UnknownInput(name) => write!(formatter, "unknown input `{name}`"),
            Self::InvalidInputType(name) => {
                write!(formatter, "input `{name}` has invalid JSON type")
            }
            Self::MissingApiKey(env, provider) => {
                write!(formatter, "{env} is required for provider {provider}")
            }
            Self::MissingEndpoint(env, provider) => {
                write!(formatter, "{env} is required for provider {provider}")
            }
            Self::UnsupportedProvider(provider) => write!(
                formatter,
                "unsupported RIG provider {provider}; supported providers: openai, openai-compatible, openai-responses, anthropic, ollama, openrouter, deepseek, xai, mock"
            ),
            Self::Provider(message) => write!(formatter, "RIG LLM generation failed: {message}"),
        }
    }
}

impl Error for RigError {}

const fn source_digest(source: &[u8]) -> u64 {
    let mut digest = 0xcbf2_9ce4_8422_2325_u64;
    let mut index = 0;
    while index < source.len() {
        digest ^= source[index] as u64;
        digest = digest.wrapping_mul(0x0000_0100_0000_01b3);
        index += 1;
    }
    digest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_provider_is_deterministic() {
        let response = execute(&Map::from_iter([
            ("provider".to_owned(), json!("mock")),
            ("model".to_owned(), json!("test-model")),
            ("prompt".to_owned(), json!("hello")),
        ]))
        .expect("mock response");

        assert_eq!(response.outputs["text"], "mock:test-model:hello");
    }

    #[test]
    fn wrong_prompt_type_fails_closed() {
        let error = execute(&Map::from_iter([
            ("provider".to_owned(), json!("mock")),
            ("model".to_owned(), json!("test-model")),
            ("prompt".to_owned(), json!(42)),
        ]))
        .expect_err("invalid prompt");

        assert!(error.to_string().contains("prompt"));
    }

    #[test]
    fn api_key_input_is_rejected_as_unknown() {
        let error = execute(&Map::from_iter([
            ("provider".to_owned(), json!("mock")),
            ("model".to_owned(), json!("test-model")),
            ("prompt".to_owned(), json!("hello")),
            ("api_key".to_owned(), json!("must-not-enter-history")),
        ]))
        .expect_err("secret input must be rejected");

        assert!(error.to_string().contains("unknown input `api_key`"));
    }

    #[test]
    fn base_url_input_is_rejected_as_unknown() {
        let error = execute(&Map::from_iter([
            ("provider".to_owned(), json!("mock")),
            ("model".to_owned(), json!("test-model")),
            ("prompt".to_owned(), json!("hello")),
            ("base_url".to_owned(), json!("https://attacker.invalid")),
        ]))
        .expect_err("endpoint input must be rejected");

        assert!(error.to_string().contains("unknown input `base_url`"));
    }
}
