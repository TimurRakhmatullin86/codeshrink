# Show HN: CodeShrink – Query-aware code compression for LLMs (1.3M LOC → 1.5k lines, 2.8s)

Hi HN,

I built CodeShrink, a CLI tool that compresses code repositories down to only the parts relevant to your question. It uses Tree-sitter AST parsing to find and rank symbols (functions, classes, types) by query relevance, then extracts just those symbols with surrounding context.

Example: `codeshrink "authentication" --path ./next.js` on the Next.js monorepo (1.29M LOC, 23k files) returns 1,509 lines in 2.8 seconds. That's 99.9% compression — only auth-related functions, their signatures, imports, and 5 lines of context.

How it works:
1. Tree-sitter parses all files (7 languages: TS, JS, Python, Rust, Go, Java, TSX)
2. Query keywords are extracted with semantic grouping (e.g., "auth" expands to login/token/jwt/session)
3. Symbols are ranked by name match + signature match + path boost + 1-hop expansion
4. Top-K symbols are extracted with configurable context windows, overlapping ranges merged
5. Output in markdown/xml/plain with file headers and line numbers

Benchmarks (real, release build, Apple Silicon):
- Express (21k LOC): 97% compression, 25ms
- Fastify (78k LOC): 98-99% compression, 70-93ms
- Next.js (1.29M LOC): 99.8-99.9% compression, 2.2-2.8s

The Rust binary is ~4MB with zero runtime dependencies. There's also an npm package via napi-rs: `npm install codeshrink`.

I wrote this because existing tools either dump the entire repo (Repomix) or require a persistent server/database (CodeGraph). I wanted something you can pipe directly into an LLM prompt: `codeshrink "where is auth?" | pbcopy`.

Code: https://github.com/timurrus/codeshrink
License: MIT/Apache-2.0
