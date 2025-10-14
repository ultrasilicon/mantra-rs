use crate::prompts::Prompts;
use anyhow::{Context, Result};
use async_openai::{
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestSystemMessageContent, ChatCompletionRequestUserMessage,
        ChatCompletionRequestUserMessageContent, CreateChatCompletionRequestArgs,
    },
    Client,
};

pub struct DeveloperAgent<'a> {
    pub model: &'a str,
    pub client: Client<async_openai::config::OpenAIConfig>,
}

impl<'a> DeveloperAgent<'a> {
    pub fn new(model: &'a str) -> Self {
        Self {
            model,
            client: Client::new(),
        }
    }

    /// Produce **entire updated file** using the Dev prompt.
    pub async fn generate(
        &self,
        original_file_path: &str,
        original_file_content: &str,
        analysis: &str,
        fewshot: &[String],
        refactor_prompt: &str,
    ) -> Result<String> {
        let system = Prompts::developer();
        let fewshot_blob = if fewshot.is_empty() {
            String::new()
        } else {
            format!("\n\n### Few-shot examples:\n{}", fewshot.join("\n\n---\n"))
        };

        let user_input = format!(
            r#"### Context

* Target file path: {path}
* Static analysis (excerpt):
  {analysis}

### Code to refactor

```rust
{code}
```

### Refactoring Request

{req}

{few}
"#,
            path = original_file_path,
            analysis = analysis,
            code = original_file_content,
            req = refactor_prompt,
            few = fewshot_blob
        );

        let req = CreateChatCompletionRequestArgs::default()
            .model(self.model)
            .messages([
                ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                    content: ChatCompletionRequestSystemMessageContent::Text(system.to_string()),
                    name: None,
                }),
                ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(user_input),
                    name: None,
                }),
            ])
            .temperature(0.0)
            .build()?;

        let resp = self.client.chat().create(req).await?;
        let content = resp.choices[0].message.content.clone().unwrap_or_default();
        let file = crate::agents::extract_rust_block(&content)
            .context("DeveloperAgent: no Rust code block found in response")?;
        Ok(file)
    }
}

/// Utility shared by agents to pull the `rust ...` block.
pub fn extract_rust_block(s: &str) -> Option<String> {
    let fence = "`rust";
    let start = s.find(fence)?;
    let rest = &s[start + fence.len()..];
    let end = rest.find("`")?;
    Some(rest[..end].trim_start_matches('\n').trim().to_string())
}
