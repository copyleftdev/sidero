use anyhow::{Context, Result};
use tokio::process::Command;
use tokio::io::AsyncWriteExt;
use serde_json::Value;
use tempfile::NamedTempFile;

pub struct SemgrepWrapper;

impl SemgrepWrapper {
    pub async fn get_version() -> Result<String> {
        let output = Command::new("semgrep")
            .arg("--version")
            .output()
            .await
            .context("Failed to execute semgrep --version")?;

        let output_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(output_str)
    }

    pub async fn get_supported_languages() -> Result<Vec<String>> {
        let output = Command::new("semgrep")
            .args(["show", "supported-languages"])
            .output()
            .await
            .context("Failed to execute semgrep show supported-languages")?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let languages = output_str
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(languages)
    }

    pub async fn scan(config: Option<String>, paths: Vec<String>) -> Result<Value> {
        let mut cmd = Command::new("semgrep");
        cmd.arg("scan")
           .arg("--json")
           .arg("--experimental");
        
        if let Some(cfg) = config {
            cmd.arg("--config").arg(cfg);
        }

        // Add paths
        for path in paths {
            cmd.arg(path);
        }

        let output = cmd.output().await.context("Failed to execute semgrep scan")?;
        
        if !output.status.success() {
             // Try to parse stdout/stderr even if it failed, sometimes semgrep returns findings with non-zero exit code
             // But usually non-zero means error in execution for simple invocations.
             // However, for MCP, we might want to return the stderr as error.
             if output.stdout.is_empty() {
                 let stderr = String::from_utf8_lossy(&output.stderr);
                 anyhow::bail!("Semgrep failed: {}", stderr);
             }
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: Value = serde_json::from_str(&stdout).context("Failed to parse Semgrep JSON output")?;
        
        Ok(json)
    }

    pub async fn scan_with_custom_rule(rule_content: String, code_files: Vec<String>) -> Result<Value> {
        let rule_file = NamedTempFile::new().context("Failed to create temp rule file")?;
        let rule_path = rule_file.path().to_str().unwrap().to_string();
        
        // Write content - we need async writing or just standard sync write since it's small/local
        // For simplicity and since NamedTempFile is sync, we use std::fs
        std::fs::write(&rule_path, rule_content).context("Failed to write rule content")?;

        let mut cmd = Command::new("semgrep");
        cmd.arg("scan")
           .arg("--json")
           .arg("--experimental")
           .arg("--config")
           .arg(&rule_path);

        for path in code_files {
            cmd.arg(path);
        }

        let output = cmd.output().await.context("Failed to execute semgrep scan")?;

        if !output.status.success() {
             if output.stdout.is_empty() {
                 let stderr = String::from_utf8_lossy(&output.stderr);
                 anyhow::bail!("Semgrep failed: {}", stderr);
             }
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: Value = serde_json::from_str(&stdout).context("Failed to parse Semgrep JSON output")?;
        
        Ok(json)
    }

    pub async fn dump_ast(code: String, language: String) -> Result<Value> {
        let code_file = NamedTempFile::new().context("Failed to create temp code file")?;
        let code_path = code_file.path().to_str().unwrap().to_string();
        std::fs::write(&code_path, code).context("Failed to write code content")?;

        let output = Command::new("semgrep")
            .arg("--dump-ast")
            .arg("--json")
            .arg("--experimental")
            .arg("--lang")
            .arg(language)
            .arg(&code_path)
            .output()
            .await
            .context("Failed to execute semgrep --dump-ast")?;

        if !output.status.success() {
             let stderr = String::from_utf8_lossy(&output.stderr);
             anyhow::bail!("Semgrep AST dump failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: Value = serde_json::from_str(&stdout).context("Failed to parse Semgrep AST output")?;
        Ok(json)
    }
}
