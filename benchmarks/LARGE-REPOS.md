# Large Repo Benchmarks

Machine: Apple Silicon, release build (`cargo build --release`), rayon parallel parsing.
Date: 2026-09-07

## Express (21,475 LOC, 141 files)

| Query | Input LOC | Output LOC | Compression | Symbols found/returned | Latency |
|---|---|---|---|---|---|
| "middleware" | 21,475 | 628 | 97.1% | 1,984 → 50 | 28.3ms |
| "routing" | 21,475 | 655 | 96.9% | 1,984 → 50 | 25.3ms |
| "error handling" | 21,475 | 655 | 96.9% | 1,984 → 50 | 20.6ms |

## Fastify (77,959 LOC, 294 files)

| Query | Input LOC | Output LOC | Compression | Symbols found/returned | Latency |
|---|---|---|---|---|---|
| "route handler" | 77,959 | 1,454 | 98.1% | 9,102 → 50 | 92.7ms |
| "validation" | 77,959 | 1,222 | 98.4% | 9,102 → 50 | 71.9ms |
| "plugin" | 77,959 | 721 | 99.1% | 9,102 → 50 | 67.5ms |

## Next.js (1,294,421 LOC, 23,049 files)

| Query | Input LOC | Output LOC | Compression | Symbols found/returned | Latency |
|---|---|---|---|---|---|
| "server action" | 1,294,421 | 1,509 | 99.9% | 121,401 → 50 | 2,822.8ms |
| "routing" | 1,294,421 | 2,655 | 99.8% | 121,401 → 50 | 2,757.1ms |
| "middleware" | 1,294,421 | 1,709 | 99.9% | 121,401 → 50 | 2,252.5ms |

## Key observations

- Latency scales linearly with file count: ~0.1ms/file (I/O + tree-sitter parse).
- Rayon parallel parsing: 3× speedup on Next.js (8.7s → 2.8s).
- Compression ratio improves with repo size: 97% (21k LOC) → 99.9% (1.29M LOC).
- All queries complete under 3 seconds even on a 1.29M LOC monorepo.
