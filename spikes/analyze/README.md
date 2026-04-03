# Spike de analisis estatico e historico

Objetivo: definir el stack Rust mas solido para analizar repositorios de forma reproducible, resumible y barata de recalcular.

## Decision

La recomendacion para esta etapa es:

- `gix` como base para leer repositorios, recorrer arboles y preparar historia Git.
- `ignore` solo como apoyo para recorridos sobre working trees cuando haga falta respetar `.gitignore`.
- `memchr` y `bstr` para escaneo de bytes, conteo de lineas y manejo de texto no UTF-8.
- `infer` para un fast-path de firma de archivo cuando queramos clasificar binarios conocidos.
- `gengo` como capa principal de deteccion de lenguajes a nivel de repositorio.
- `tree-sitter` y `tree-sitter-rust` solo para enriquecimiento semantico opcional, no para la estadistica base.
- `rayon` para paralelizar el trabajo CPU-bound por archivo o por commit.
- `serde` para serializar snapshots y checkpoints.

## Why this stack

`gix` encaja mejor con `pulse` que `git2` para una implementacion Rust-native orientada a escala. Su ecosistema cubre repo, traversal, diff y commit-graph, lo que reduce la cantidad de piezas ad-hoc en el analisis historico.

Para inventario y estadisticas de archivos, el camino mas estable es tratar el arbol Git como fuente de verdad. El filesystem solo debe ser fallback operativo. Eso evita ruido de archivos locales no versionados y alinea el analisis con el revision scope real.

Para conteo de lineas y lectura de blobs, el camino mas barato es byte-oriented. `memchr` da primitives optimizadas para buscar bytes en `&[u8]`, y `bstr` aporta ergonomia cuando el contenido no es UTF-8 valido.

Para lenguajes, `gengo` es el candidato mas cercano a GitHub Linguist porque trabaja con colecciones de archivos y soporta repositorios Git, incluyendo atributos Git y repos bare. Eso lo hace mas util que un detector puramente por extension.

## Spike scope

Este spike cubre la parte que despues alimenta `pulse run`:

- inventario de archivos y directorios por revision
- tamano por archivo
- conteo de lineas por archivo y agregado
- clasificacion binario/texto
- deteccion de lenguaje por archivo y por repositorio
- timestamps de primera/ultima aparicion en historia
- preparacion para recorridos de commits, weekly buckets y file history

## Non-goals

- UI
- exportes avanzados
- semantic analysis generalista para todos los lenguajes
- benchmarks definitivos de rendimiento

## Suggested implementation shape

1. Abrir el repo con `gix`.
2. Resolver la revision objetivo y caminar el tree de Git.
3. Para cada blob texto:
   - calcular bytes, line count y heuristica de binario/texto
   - detectar lenguaje via `gengo`
4. Registrar snapshots en estructuras serializables con `serde`.
5. Para historia:
   - usar traversal de commits desde `gix`
   - agrupar por semana
   - persistir progreso incremental por commit procesado

## Known risks

- `gengo` es la mejor apuesta funcional para lenguaje, pero conviene validarlo con un corpus propio antes de congelarlo.
- `tree-sitter` no deberia cargarse como dependencia principal de V1 para language stats; el coste de grammars por lenguaje crece rapido.
- `git2` sigue siendo una alternativa razonable si aparece una necesidad concreta de compatibilidad con libgit2, pero no lo tomararia como base inicial.

