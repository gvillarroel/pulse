# Spike: métricas, estadísticas y almacenamiento resumible en Rust

Fecha: 2026-04-02

## Objetivo

Definir una base sólida para:

- métricas de repositorio
- métricas de contribuyentes
- agregados semanales de evolución
- persistencia resumible por etapa

## Recomendación

La combinación más razonable para esta capa es:

- `rayon` para paralelismo CPU-bound entre repositorios
- `rusqlite` con SQLite para checkpoints, snapshots y reanudación
- `serde` para serializar checkpoints y artefactos intermedios
- `csv` para exportes simples de snapshots y agregados
- `time` para timestamps y ventanas semanales
- consumir como input los outputs del spike de análisis, que ya resuelve árbol Git, conteo y lenguaje

## Por qué

### Estadísticas y agregación

`rayon` es suficiente para paralelizar trabajo por repositorio o por lote de registros sin meter un runtime async innecesario. Para este problema el cuello suele ser CPU + I/O local, no un servidor de larga vida.

### Persistencia resumible

`rusqlite` con SQLite es la mejor opción inicial para checkpoints y snapshots:

- archivo único dentro de `--state-dir`
- fácil de inspeccionar y respaldar
- transacciones simples para marcar etapas `pending/running/completed/failed`
- esquema relacional natural para repositorios, snapshots, contribuyentes y buckets semanales

### Exportes

`serde` y `csv` cubren el caso práctico del spike: guardar resultados intermedios, reabrirlos en una segunda corrida y exportar tablas estables para validación manual o automatizada.

## Spike propuesto

Construir un prototipo mínimo que haga esto sobre 1 o 2 repositorios de prueba:

1. leer los outputs del spike de análisis para un repo ya normalizado
2. calcular métricas de repositorio, contribuidor y bucket semanal
3. persistir cada etapa en SQLite con checkpoint por repositorio
4. exportar una vista CSV y una vista JSON para validar estabilidad de esquema
5. reanudar una segunda ejecución sin recomputar etapas ya completas

## Esquema mínimo de SQLite

- `runs`
- `repositories`
- `repo_stage_checkpoints`
- `repo_snapshots`
- `file_snapshots`
- `contributor_snapshots`
- `weekly_evolution`
- `exports`

## Riesgos

- la capa de stats no debe duplicar la responsabilidad del spike de análisis, o terminaremos con dos implementaciones del mismo inventario.
- SQLite funcionará bien, pero hay que cuidar escrituras por lote para no convertirlo en cuello de botella.

## Conclusión

Para el spike de métricas/estadísticas/storage, la apuesta que mejor equilibra control, resumabilidad y mantenibilidad es:

- `rayon` + `rusqlite` + `serde` + `csv` + `time`

Y dejar el árbol Git, la detección de lenguaje y el conteo base en el spike de análisis, no en esta capa de persistencia y agregación.
