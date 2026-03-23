use serde::{Deserialize, Serialize};
use crate::domain::redaction::RedactionEngine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellContext {
    pub shell_type: String,
    pub os: String,
    pub cwd: String,
    pub recent_commands: Vec<CommandEntry>,
    pub env_snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEntry {
    pub command: String,
    pub exit_code: i32,
    pub output_preview: Option<String>,
}

pub fn build_system_prompt(context: &ShellContext, redaction: &RedactionEngine) -> String {
    let mut prompt = format!(
        "You are a terminal command assistant. Help the user with shell commands.\n\n\
         Context:\n\
         - Shell: {}\n\
         - OS: {}\n\
         - Current directory: {}\n",
        context.shell_type, context.os, context.cwd,
    );

    if !context.recent_commands.is_empty() {
        prompt.push_str("- Recent commands:\n");
        for cmd in &context.recent_commands {
            let redacted_cmd = redaction.redact(&cmd.command);
            prompt.push_str(&format!(
                "  $ {} (exit: {})\n",
                redacted_cmd, cmd.exit_code
            ));
            if let Some(preview) = &cmd.output_preview {
                let redacted_output = redaction.redact(preview);
                prompt.push_str(&format!("    {}\n", redacted_output));
            }
        }
    }

    prompt.push_str("\nRules:\n\
         - Provide exact commands the user should run\n\
         - Be concise and direct\n\
         - For dangerous commands (rm -rf, sudo, etc.), warn the user\n\
         - Prefer portable POSIX commands when possible\n");

    prompt
}

pub fn build_error_diagnosis_prompt(
    command: &str,
    output: &str,
    exit_code: i32,
    context: &ShellContext,
    redaction: &RedactionEngine,
) -> String {
    let redacted_cmd = redaction.redact(command);
    let redacted_output = redaction.redact(output);

    format!(
        "A command failed in the terminal. Diagnose the error and suggest a fix.\n\n\
         Shell: {}\n\
         OS: {}\n\
         Working directory: {}\n\n\
         Command: {}\n\
         Exit code: {}\n\
         Output:\n```\n{}\n```\n\n\
         Provide:\n\
         1. What went wrong (brief)\n\
         2. How to fix it (specific command or action)\n",
        context.shell_type, context.os, context.cwd,
        redacted_cmd, exit_code, redacted_output
    )
}
