![ELEN_GALAXIA Hero](assets/hero.png)

# 🌌 ELEN_GALAXIA: Personal Edge Architecture

> **Global SOTA 2025/2026 Configuration & Strategy**
> **Objective:** Zero-Hardware Overhead Architecture for Universal Intelligence.

---

## 🏛️ Project Vision

**ELEN_GALAXIA** is the extraction of a master infrastructure strategy. It provides a "Hardware-Zero" blueprint where your local machine acts only as a thin-client (IDE), while all state, memory, and high-performance compute live on the Global Edge.

[**📖 Read the Full Architecture Guide**](ARCHITECTURE.md)

---

## 🚀 Key Features

- **⚡ Hardware-Zero Execution:** Shift compute-heavy tasks (AI, DB, Vector Search) to Cloudflare.
- **🧠 Global Memory:** Shared vector storage via Cloudflare Vectorize.
- **🔗 MCP Native:** Pre-configured Model Context Protocol servers for zero-token discovery.
- **🛡️ Dark Mode Security:** Zero Trust infrastructure with no public ingress required.

---

## 📊 Comparison: Legacy vs. SOTA

| Component | Local (Legacy) | **ELEN_GALAXIA (Edge)** |
|-----------|----------------|----------------------------|
| **Database** | SQLite (Local File) | **Cloudflare D1** (Serverless) |
| **Vectors** | Docker / Pinecone | **Cloudflare Vectorize** |
| **AI Models** | Local GGUF (8GB+ RAM) | **Workers AI** (Edge Native) |
| **Compute** | Node/Python Process | **Cloudflare Workers (WASM)** |

---

## 📁 Repository Structure

```text
.
├── global-config/           # 💎 THE SOURCE OF TRUTH
│   ├── mcp_registry.json    # Standardized MCP Tools
│   └── secrets.json         # (Ignored) Sensitive credentials
│
├── servers/                 # 🛰️ Local & Edge MCP Wrappers
│   └── agent-cgi/           # Rust-based CGI for Agents
│
└── mcp-gateway/             # 🌉 Bridge between IDE and Edge
```

---

## 🛠️ The 2026 SOTA Stack

We utilize a hybrid reasoning pipeline that preserves context efficiency while maximizing exploratory depth:

```text
≅(Ψ├[→CoT 🔄ReAct ↻∆ ♻️Verify]⛓️∞Output PureEnhanced)
```

- **Ψ (Quantum Branching):** Exploratory reasoning paths.
- **⛓️∞ (Orchestration):** Recursive self-improvement loops.
- **📦~ (LLMLingua-2):** 25x hybrid context compression.

---

## 🏁 Getting Started

1. **Clone & Bind:**

   ```bash
   git clone https://github.com/LERMF/ELEN_GALAXIA.git
   ```

2. **Setup Credentials:**
   Copy `global-config/secrets.json.template` to `global-config/secrets.json` and fill in your Cloudflare Keys.
3. **Register MCP:** Point your AgentIC IDE to the `global-config/mcp_registry.json` path.

---

## 📜 License

Distribute freely under the **MIT License**. Created by [LERMF](https://github.com/LERMF).
