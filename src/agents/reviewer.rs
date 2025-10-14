use anyhow::Result;
use async_openai::{
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestSystemMessageContent, ChatCompletionRequestUserMessage,
        ChatCompletionRequestUserMessageContent, CreateChatCompletionRequestArgs,
    },
    Client,
};
use serde::Deserialize;

use crate::config::RefactorKind;
use crate::{prompts::Prompts, verification::verify_refactor_heuristic};

#[derive(Debug, Deserialize)]
pub struct ReviewVerdict {
    pub verdict: String,
    pub reasons: Option<Vec<String>>,
    pub patch_guidance: Option<String>,
    pub checklist: Option<serde_json::Value>,
}

pub struct ReviewerAgent<'a> {
    pub model: &'a str,
    pub client: Client<async_openai::config::OpenAIConfig>,
}

impl<'a> ReviewerAgent<'a> {
    pub fn new(model: &'a str) -> Self {
        Self {
            model,
            client: Client::new(),
        }
    }

    pub async fn review(
        &self,
        kind: RefactorKind,
        original: &str,
        candidate: &str,
        analysis: &str,
        fmt_clean: bool,
        clippy_clean: bool,
        compiler_logs: Option<&str>,
    ) -> Result<ReviewVerdict> {
        // quick local gate first
        let verified = verify_refactor_heuristic(kind.clone(), original, candidate);

        let system = Prompts::reviewer();
        let user_input = format!(
            r#"### Inputs
Original:

```rust
{original}
```

Candidate:

```rust
{candidate}
```

Static analysis (excerpt):
{analysis}

fmt_clean={fmt_clean}  clippy_clean={clippy_clean}

Compiler/test logs (optional):
{logs}

Refactor kind: {kind:?}
"#,
            original = original,
            candidate = candidate,
            analysis = analysis,
            fmt_clean = fmt_clean,
            clippy_clean = clippy_clean,
            logs = compiler_logs.unwrap_or("<none>"),
            kind = kind
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
        let json_text = content
            .split("```json")
            .nth(1)
            .and_then(|s| s.split("```").next())
            .map(|s| s.trim())
            .unwrap_or(content.trim());

        let mut verdict: ReviewVerdict = serde_json::from_str(json_text).or_else(|_: serde_json::Error| {
            Ok::<ReviewVerdict, serde_json::Error>(ReviewVerdict {
                verdict: if verified {
                    "accept".into()
                } else {
                    "revise".into()
                },
                reasons: Some(vec!["LLM JSON parse fallback".into()]),
                patch_guidance: None,
                checklist: None,
            })
        })?;

        if !verified && verdict.verdict == "accept" {
            verdict.verdict = "revise".into();
            verdict
                .reasons
                .get_or_insert(vec![])
                .push("Local heuristic failed to verify refactor".into());
        }

        Ok(verdict)
    }
}
