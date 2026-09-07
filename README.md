# CodeShrink

**Query-aware AST code compression for LLMs. 1.29M LOC → 1,509 lines in 2.8s.**

Tree-sitter powered, not embeddings. Ask a question, get only the code that matters.

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-MIT)

![CodeShrink demo](demo.gif)

## The Problem

You paste your entire repo into an LLM. It costs $3, takes 80k tokens, and the model loses the answer somewhere in the middle. Truncation cuts the wrong files. Vector RAG returns 8k tokens of tangentially related chunks.

## The Solution

```bash
codeshrink "where is authentication?" --path ./my-app
```

```
# CodeShrink: query="authentication"

## src/auth/middleware.ts

### `authenticate` (score: 1.00)

   3 | export function authenticate(req: Request): boolean {
   4 |     const token = req.headers.authorization;
   5 |     if (!token) return false;
   6 |     return verifyToken(token);
   7 | }
```

**3,480 lines → 108 lines. 96.9% compression. 14.5ms.**

## How It Works

1. **SCAN** — Tree-sitter parses every file into an AST (parallel via rayon), extracting function/class/type definitions
2. **MATCH** — Your query is split into terms with semantic grouping ("auth" → also matches "login", "token", "jwt")
3. **RANK** — Symbols are scored by name match + path relevance + 1-hop signature expansion
4. **EXTRACT** — Top-K symbols with configurable context lines, merged when overlapping
5. **OUTPUT** — Formatted as Markdown, XML, or plain text with line numbers

No embeddings. No vector database. No API keys. Pure AST symbol matching.

## Install

### Rust CLI

```bash
cargo install codeshrink
```

### Node.js

```bash
npm install codeshrink
```

## Usage

### CLI

```bash
# Basic query
codeshrink "where is the database connection?" --path ./my-project

# Narrow context (2 lines instead of default 5)
codeshrink "error handling" -c 2

# XML output (for LLM system prompts)
codeshrink "routing" --format xml

# Pipe into an LLM
codeshrink "auth" --path ./app | pbcopy

# Stats only
codeshrink "auth" --stats-only
```

### Node.js API

```js
const { shrink } = require('codeshrink');

const result = shrink('authentication', '/path/to/repo', {
    contextLines: 5,
    maxSymbols: 50,
    format: 'markdown',
});

console.log(result.compressed);
console.log(result.stats);
// { filesScanned: 141, filesMatched: 14, symbolsFound: 1984,
//   symbolsReturned: 50, inputLines: 21475, outputLines: 628,
//   compressionRatio: 0.971 }
```

### Rust API

```rust
use codeshrink_core::{shrink, ShrinkOptions};
use std::path::Path;

let result = shrink(
    "where is authentication?",
    Path::new("./my-project"),
    &ShrinkOptions::default(),
)?;

println!("{}", result.compressed);
```

## Benchmarks (measured, release build, Apple Silicon)

| Repo | Query | Input LOC | Output LOC | Compression | Latency |
|---|---|---|---|---|---|
| Express (21k LOC) | "middleware" | 21,475 | 628 | 97.1% | 28ms |
| Express (21k LOC) | "routing" | 21,475 | 655 | 96.9% | 25ms |
| Fastify (78k LOC) | "route handler" | 77,959 | 1,454 | 98.1% | 93ms |
| Fastify (78k LOC) | "plugin" | 77,959 | 721 | 99.1% | 68ms |
| Next.js (1.29M LOC) | "server action" | 1,294,421 | 1,509 | 99.9% | 2,823ms |
| Next.js (1.29M LOC) | "middleware" | 1,294,421 | 1,709 | 99.9% | 2,253ms |

## Supported Languages

TypeScript, TSX, JavaScript, Python, Rust, Go, Java (via tree-sitter).

## vs Alternatives

| | CodeShrink | Repomix | CodeGraph | Truncation | RAG |
|---|---|---|---|---|---|
| Query-aware | **Yes** | No (full dump) | Yes (MCP) | No | Partial |
| Standalone CLI | **Yes** | Yes | No (MCP server) | N/A | No |
| Latency (78k LOC) | **93ms** | ~500ms | ~2s | 0ms | ~200ms |
| Output (78k LOC) | **1.4k lines** | 78k lines | ~2k lines | 20k tokens | ~8k tokens |
| Dependencies | **0** (single binary) | Node.js | Python + SQLite | N/A | Embeddings model |
| npm package | **Yes** (napi-rs) | Yes | No | N/A | Varies |

## License

MIT OR Apache-2.0
