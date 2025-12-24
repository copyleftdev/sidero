# 🤖 Sidero

![Sidero Mascot](media/image.png)

> **"Iron-Clad Security for the Modern Stack."**

[![Rust](https://img.shields.io/badge/built_with-Rust-dca282.svg)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-Compatible-green)](https://modelcontextprotocol.io)
[![Status](https://img.shields.io/badge/Status-Verified-blue)](https://github.com/copyleftdev/sidero)

**Sidero** (Greek: *Iron*) is a blazing-fast, Rust-based Model Context Protocol (MCP) server for [Semgrep](https://semgrep.dev). It acts as a lightweight, memory-safe, and asynchronous bridge between your LLM workspace (Claude, Cursor, etc.) and the powerful Semgrep static analysis engine.

Unlike existing wrappers, Sidero is built for speed (`tokio`), correctness, and "batteries-included" feature parity with the official Python implementation, but with the raw power of Rust.

---

## 🌟 Features

*   **⚡ Zero-Latency Startup**: compiled binary vs Python interpreter overhead.
*   **🛡️ Rust Reliability**: Type-safe, memory-safe, and concurrent.
*   **🔍 Full Feature Parity**:
    *   **Scanning**: Run standard Semgrep scans on your codebase.
    *   **Custom Rules**: Prompt your LLM to write a custom rule, and Sidero will run it immediately.
    *   **AST Dumps**: Inspect the raw Abstract Syntax Tree of your code for deep debugging.
    *   **Findings**: Fetch your historical security findings directly from Semgrep App.
*   **📦 Resources & Prompts**: Built-in prompts to help LLMs write better security rules.

## 🚀 Installation

### Prerequisites
*   [Rust Toolchain](https://rustup.rs/) (cargo)
*   [Semgrep CLI](https://semgrep.dev/docs/getting-started/) (`semgrep` must be in your PATH)

### Build
```bash
git clone https://github.com/copyleftdev/sidero
cd sidero
cargo build --release
```

The binary will be waiting for you at `./target/release/sidero`.

## ⚙️ Configuration

To use Sidero, add it to your MCP client configuration (e.g., `claude_desktop_config.json`).

> 🔑 **Note:** To use `semgrep_findings`, you must provide your `SEMGREP_APP_TOKEN`.

```json
{
  "mcpServers": {
    "sidero": {
      "command": "/absolute/path/to/sidero/target/release/sidero",
      "args": [],
      "env": {
        "SEMGREP_APP_TOKEN": "your-semgrep-app-token-here"
      }
    }
  }
}
```

## 🛠️ Usage

Once connected, your LLM will have access to these tools:

| Tool | Description |
| :--- | :--- |
| **`semgrep_scan`** | Scan specific files or directories with a config (e.g., "p/security-audit"). |
| **`semgrep_scan_with_custom_rule`** | Execute an ad-hoc YAML rule on provided code files. |
| **`get_abstract_syntax_tree`** | Dump the AST of a code snippet for language-level analysis. |
| **`semgrep_findings`** | Retrieve findings from your Semgrep Dashboard (SAST, SCA, Secrets). |

### Example Prompts
*   *"Scan `src/main.rs` for security vulnerabilities using the default ruleset."*
*   *"Write a Semgrep rule to detect `unwrap()` calls in Rust and run it on this file."*
*   *"Show me the critical vulnerabilities from my dashboard."*

## 🧪 Advanced Usage (JSON-RPC)

You can interact with Sidero directly via standard input if you are building your own client or debugging.

**Note:** When sending multi-line rules via JSON, ensure proper escaping of newlines (`\n`).

```bash
echo '{"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "semgrep_scan_with_custom_rule", "arguments": {"code_files": ["app.js"], "rule": "rules:\n  - id: test-eval\n    patterns:\n      - pattern: eval(...)\n    message: \"Eval found!\"\n    languages: [javascript]\n    severity: ERROR"}}}' | ./target/release/sidero
```

## 🏗️ Architecture

Sidero leverages:
*   **`tokio`**: For async runtime and non-blocking I/O.
*   **`serde_json`**: For high-performance JSON-RPC serialization.
*   **`reqwest`**: For communicating with the Semgrep.dev API.

---

*Built with ❤️ in Rust.*
