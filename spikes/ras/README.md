# RAS Spike

Este directorio concentra el spike inicial para la arquitectura Rust de `pulse`.

Objetivos de este spike:

- validar el stack de librerías por etapa del pipeline
- separar responsabilidades en spikes pequeños y compilables
- dejar criterios claros para decidir entre alternativas como `gix` y `git` CLI
- probar piezas nucleares de resumibilidad, análisis y persistencia

## Recomendación de stack

### Fetch y actualización de repositorios

- `gix = 0.81.0`: mejor candidato como backend Git nativo en Rust para lectura, revision walking y, a medio plazo, operaciones de fetch sin depender de procesos externos
- `git` CLI: backend de arranque recomendado para clone/fetch por robustez operativa y paridad inmediata con workflows reales
- `serde = 1.0.228`: serialización de checkpoints y metadatos
- `anyhow = 1.0.102`: errores ergonómicos para el spike

Decisión inicial:

- usar `git` CLI como implementación V0 para descarga y actualización
- mantener `gix` como alternativa evaluada y preferida para lectura intensiva e historia
- no depender de `git2/libgit2` salvo que aparezca un gap concreto; hoy añade una dependencia C que no encaja con el objetivo de despliegue simple

### Análisis estático de working tree

- `gix = 0.81.0`: lectura Git e historia, sin meter `git2`
- `ignore = 0.4.25`: traversal respetando `.gitignore`
- `jwalk = 0.8.1`: alternativa a evaluar para scans masivos en paralelo
- `memchr = 2.7.6`: conteo rápido de `\n` y búsqueda byte-oriented
- `bstr = 1.12.1`: manejo ergonómico de bytes no UTF-8
- `infer = 0.19.0`: complemento por firmas para archivos binarios conocidos
- `gengo = 0.8.1`: detección de lenguaje con mejor encaje repo-level
- `rayon = 1.11.0`: paralelismo CPU-bound para archivos y lotes
- `tokei = 14.0.0`: baseline rápida para contraste de LOC por lenguaje
- `content_inspector = 0.2.4`: complemento heurístico simple para texto/binario

Decisión inicial:

- `gix` para lectura e historia; `ignore` como default para walking del árbol
- `jwalk` sólo como benchmark complementario; no como default hasta validar impacto real con repos grandes
- `memchr` + `bstr` como núcleo del escaneo byte-oriented
- `gengo` como apuesta principal para lenguaje; `tokei` queda como referencia comparativa rápida
- `content_inspector` como heurística simple y `infer` sólo para enriquecer tipo de binarios
- `tree-sitter` queda explícitamente fuera del camino principal de V1

### Métricas, evolución y persistencia

- `rusqlite = 0.39.0`: mejor default para checkpoints y metadatos locales en V1
- `time = 0.3.44`: buckets semanales y timestamps en la ruta recomendada
- `csv = 1.4.0`: exportes estables y simples para validación y datasets
- `rayon = 1.11.0`: paralelismo por repositorio o lote de agregación
- `serde = 1.0.228`: export serializable y blobs de config
- `gix = 0.81.0`: lectura de commits y evolución semanal

Decisión inicial:

- SQLite con `rusqlite` como store transaccional local
- series semanales calculadas desde Git y persistidas incrementalmente
- `stats` consume outputs normalizados de `analyze`; no duplica inventario de archivos
- evitar meter `polars` o motores dataframe al núcleo V1; sumarían peso sin resolver el core de resumibilidad

## Estructura

- `fetch-spike`: criterios y utilidades para descarga resumible
- `analyze-spike`: inventario estático, clasificación y conteo
- `stats-spike`: buckets semanales y persistencia SQLite
- `alternatives.md`: inventario completo de opciones razonables y cobertura real
- `reporting-spike.md`: evaluación de librerías y formato para reportes HTML interactivos autocontenidos
- `../fetch`: documentación del worker de fetch
- `../analyze`: documentación del worker de análisis
- `../stats`: documentación del worker de métricas y storage

## Cómo validar

```powershell
cargo test --manifest-path spikes/ras/Cargo.toml
```

## Fuentes consultadas

- `gix` / `gitoxide`
- `ignore`
- `tokei`
- `content_inspector`
- `rusqlite`

La recomendación consolidada final quedará alineada además con los resultados de los workers paralelos.
