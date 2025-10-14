pub struct Prompts;

impl Prompts {
    /// Developer agent (paper’s prompt, adapted for Rust).
    pub fn developer() -> &'static str {
        r#"### Task: Code Refactoring Based on a Specified Refactoring Type (Rust)

### Instructions — Step-by-Step
Step 1: Code Analysis.
Summarize the specific Rust code segment to be refactored and its role in the file/crate.

Step 2: Refactoring Method Reference.
Given a short list of similar refactoring examples (few-shot RAG), extract up to three applicable patterns.

Step 3: Structure Information Extraction.
Use the provided static analysis of the repository (module layout, item graph, call sites, impl blocks, traits, visibility) to gather any extra context required.

Step 4: Refactoring Execution.
Produce the **entire updated file content** implementing the refactor. Preserve semantics and public API unless explicitly requested. Keep idiomatic Rust style.

### Output contract
Return ONLY a single fenced code block with the *full file content* after refactor:
```rust
// <file.rs>
<entire updated file>
````

No extra commentary before or after the code block.
"#
    }

    /// Reviewer agent: verifies refactoring type + style + build/tests.
    pub fn reviewer() -> &'static str {
        r#"### Role: Reviewer Agent (Rust)

You are reviewing a candidate refactoring patch.

Goals (in order):

1. **Refactoring verification**: Did the intended refactor actually happen, and is it coherent?

   * ExtractMethod: a new `fn`/method exists; original logic replaced with a call or moved; names are descriptive; visibility minimal.
   * InlineMethod: removed a previously separate `fn`; call sites replaced; no dead code left.
   * MoveMethod: functionality relocated to a more appropriate impl/trait/module; imports/paths updated.
   * RenameMethod: consistent rename across definitions, trait impls, and call sites (including re-exports).
2. **Style/readability**: idiomatic Rust; runs `rustfmt` cleanly; avoid `.clone()` where unnecessary; prefer borrowing; good names and docs where helpful.
3. **Safety/correctness**: ownership/borrowing sane; no UB; no changed external behavior unless explicitly requested.

### Inputs

* Original file, Candidate file
* Static analysis summary
* Optional compiler/test logs

### Output contract

Return a short structured JSON object:

{
"verdict": "accept" | "revise",
"reasons": ["..."],
"patch_guidance": "Only if verdict=revise: concrete edits to apply",
"checklist": {
"refactor_verified": true|false,
"fmt_clean": true|false,
"clippy_clean": true|false
}
}

Keep it concise and actionable.
"#
    }

    /// Repair agent: Reflexion-style verbal RL loop on compiler/test failures.
    pub fn repair() -> &'static str {
        r#"### Role: Repair Agent (Rust) — Reflexion-style

You are given:

* The current (candidate) file content
* Compiler/test ERROR logs (fresh)
  Your job:

1. **Initial Analysis**: pinpoint root causes using exact error lines/messages.
2. **Plan**: list minimal code edits to fix errors while preserving behavior of the refactor.
3. **Act**: output the ENTIRE corrected file.

### Constraints

* Do NOT change public behavior unless obviously required to compile (e.g., trait signatures).
* Prefer borrow over clone, fix lifetimes/imports/paths/visibilities.
* Keep style idiomatic; pass `rustfmt`.

### Output contract

Respond with a single fenced code block containing the **full corrected file**:

```rust
// <file.rs>
<entire corrected file>
```

No extra commentary.
"#
    }
}
