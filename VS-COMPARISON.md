# CodeShrink vs alternatives

| Feature | CodeShrink | Repomix | CodeGraph | Truncation (20k) | RAG (embeddings) |
|---|---|---|---|---|---|
| **Query-aware** | Yes | No (full dump) | Yes (MCP) | No | Partial |
| **Latency (80k LOC)** | 93ms | ~500ms | ~2s (index + query) | 0ms | ~200ms |
| **Output size (80k LOC)** | ~1.2k lines | 50-500k lines | ~2k lines | 20k tokens | ~8k tokens |
| **Compression** | 97-99.9% | 0% | ~97% | ~75% | ~90% |
| **Dependencies** | 0 (single binary) | Node.js | Python + SQLite | N/A | Embeddings model |
| **Tree-sitter AST** | Yes | Yes (compress mode) | Yes | No | No |
| **Languages** | 7 | 30+ | 21 | Any | Any |
| **Standalone CLI** | Yes | Yes | No (MCP server) | N/A | No |
| **npm package** | Yes (napi-rs) | Yes | No | N/A | Varies |
| **Offline** | Yes | Yes | Yes | Yes | Needs model |
| **Import tracking** | Yes | No | Yes (graph) | No | No |
| **Semantic grouping** | 14 domains | No | SQL queries | No | Vector similarity |
| **1-hop expansion** | Yes (signatures) | No | Yes (graph) | No | Partial |

## When to use what

- **CodeShrink**: You have a specific question about a large codebase. You want only the relevant code, fast.
- **Repomix**: You want to dump the entire repo into one file for broad context.
- **CodeGraph**: You need persistent code graph with MCP integration in your IDE.
- **Truncation**: Your repo fits in the context window anyway.
- **RAG**: You need semantic search over documentation + code.
