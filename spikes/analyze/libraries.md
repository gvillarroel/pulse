# Matriz de librerias para analisis

## Recomendadas

| Area | Libreria | Motivo |
| --- | --- | --- |
| Recorrido de repositorio y historia | `gix` | Base Rust-native para repos, refs, traversal, diff y commit-graph. |
| Recorrido de working tree con filtros | `ignore` | Respeta `.gitignore` y filtros de forma eficiente. |
| Conteo de bytes / lineas | `memchr` | Primitivas optimizadas sobre `&[u8]` para buscar `\n` y `\0`. |
| Manejo de texto arbitrario | `bstr` | Ergonomia para bytes que no siempre son UTF-8. |
| Clasificacion binario/texto | `infer` + heuristica propia | `infer` para firmas conocidas; heuristica simple para el resto. |
| Deteccion de lenguaje | `gengo` | Soporta repos Git y atributos Git, mas cercano a Linguist. |
| Enriquecimiento semantico Rust | `tree-sitter` + `tree-sitter-rust` | Opcional, para analisis semantico posterior. |
| Paralelismo CPU-bound | `rayon` | Work-stealing simple para procesamiento por archivo/commit. |
| Snapshots y checkpoints | `serde` | Serializacion estable de estructuras de analisis. |

## Alternativas evaluadas

| Area | Alternativa | Veredicto |
| --- | --- | --- |
| Recorrido de repo | `walkdir` | Bueno para filesystem, pero insuficiente como fuente principal si queremos semantica Git. |
| Git bindings | `git2` | Maduro y util, pero menos alineado con una base Rust-native y con mas carga C/FFI. |
| Deteccion de lenguaje | `linguist` | Buena opcion de apoyo; `gengo` me parece mejor encaje para repos completos. |
| Analisis Rust | `rust-code-analysis` | Valioso para Rust y algunos lenguajes, pero demasiado limitado para el lenguaje base de todo el sistema. |
| Semantica general | `tree-sitter` | Excelente para parsing, pero no deberia ser la fuente primaria de stats de lenguaje. |

## Sources

- `gix`: https://docs.rs/gix
- `ignore`: https://docs.rs/ignore
- `memchr`: https://docs.rs/memchr
- `bstr`: https://docs.rs/bstr
- `infer`: https://docs.rs/infer
- `gengo`: https://docs.rs/gengo
- `tree-sitter`: https://docs.rs/tree-sitter
- `tree-sitter-rust`: https://docs.rs/tree-sitter-rust
- `rayon`: https://docs.rs/rayon
- `serde`: https://docs.rs/serde
- `git2`: https://docs.rs/git2
- `rust-code-analysis`: https://docs.rs/rust-code-analysis
