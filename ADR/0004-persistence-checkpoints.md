# ADR 0004: Persistencia local y checkpoints

Estado: Aceptado

Fecha: 2026-04-02

## Contexto

`pulse` necesita persistir:

- checkpoints por etapa y repositorio
- snapshots de repositorio y archivo
- agregados semanales
- metadata de runs
- exports reutilizables

La reanudación es requisito central. El sistema debe soportar:

- retries
- estados `pending/running/completed/failed/stale`
- consultas locales simples
- inspección manual del estado

## Decisión a tomar

Qué motor usar para metadata, checkpoints y snapshots base.

## Alternativas consideradas

### Opción A: `rusqlite` + SQLite

Ventajas:

- archivo único
- transacciones claras
- modelo relacional natural
- fácil inspección manual
- buen encaje con estado local y resumibilidad

Desventajas:

- dependencia SQLite
- puede requerir cuidado de batching bajo alta escritura

### Opción B: `redb`

Ventajas:

- Rust puro
- buen encaje como KV embebido
- simple para ciertos patrones

Desventajas:

- peor ajuste para consultas relacionales y snapshots multi-tabla
- más trabajo de modelado para exportes y joins

### Opción C: `duckdb`

Ventajas:

- fuerte para analytics
- SQL expresivo para datasets derivados

Desventajas:

- peor ajuste para checkpoints operativos
- resuelve más el problema de reporting que el de reanudación

### Opción D: ORM/capas async (`sqlx`, `diesel`, `sea-orm`)

Ventajas:

- ergonomía o checks extra según el caso

Desventajas:

- más peso conceptual y técnico
- mala relación coste/beneficio para un CLI batch local

## Decisión

Usar la implementación Rust de Turso/libSQL como motor principal de persistencia operativa.

Implementación concreta elegida:

- crate `libsql` para Rust
- compatible con despliegue local o remoto Turso
- modelo SQL/SQLite-compatible para checkpoints y snapshots

Alternativa descartada como baseline:

- `rusqlite` + SQLite local puro

## Razonamiento

La estructura de `pulse` es naturalmente relacional:

- repositorios
- etapas
- snapshots
- buckets semanales
- contributors

Turso/libSQL mantiene modelo SQLite-compatible y añade una ruta más flexible si luego se quiere combinar operación local, sync o despliegue remoto sin cambiar el lenguaje ni la semántica SQL base.

## Consecuencias

### Positivas

- se mantiene modelo SQL/SQLite-compatible
- abre camino a operación local y remota dentro de la misma familia tecnológica
- deja más margen para evolución futura que fijarse de entrada en SQLite local puro

### Negativas

- más complejidad operativa y de dependencia que `rusqlite`
- la ruta exacta local/remota habrá que delimitarla bien en implementación
- habrá que validar throughput real para checkpoints batch

## Validación recomendada

Medir:

- throughput de inserts/upserts por lote
- contención en escrituras concurrentes
- coste de reanudar runs grandes

Baseline:

1. `libsql`

Escalada sólo si hay problema real:

2. `rusqlite`
3. `redb`
