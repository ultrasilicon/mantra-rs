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

pub struct RepairAgent<'a> {
    pub model: &'a str,
    pub client: Client<async_openai::config::OpenAIConfig>,
}

impl<'a> RepairAgent<'a> {
    pub fn new(model: &'a str) -> Self {
        Self {
            model,
            client: Client::new(),
        }
    }

    pub async fn repair(
        &self,
        file_path: &str,
        broken_candidate: &str,
        compiler_or_test_logs: &str,
    ) -> Result<String> {
        let system = Prompts::repair();
        let user = format!(
            r#"File path: {file}

Current file content:

```rust
{code}
```

Compiler/Test errors:

````
{logs}
```"#,
            file = file_path,
            code = broken_candidate,
            logs = compiler_or_test_logs,
        );

        let req = CreateChatCompletionRequestArgs::default()
            .model(self.model)
            .messages([
                ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                    content: ChatCompletionRequestSystemMessageContent::Text(system.to_string()),
                    name: None,
                }),
                ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(user),
                    name: None,
                }),
            ])
            .temperature(0.0)
            .build()?;

        let resp = self.client.chat().create(req).await?;
        let content = resp.choices[0].message.content.clone().unwrap_or_default();
        let file = crate::agents::extract_rust_block(&content)
            .context("RepairAgent: no Rust code block found in response")?;
        Ok(file)
    }
}
