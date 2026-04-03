# Alternativas y cobertura

Fecha: 2026-04-02

Este documento responde dos preguntas:

1. qué alternativas razonables existen por etapa
2. cuáles ya quedaron cubiertas por el spike y cuáles todavía conviene probar

## Resumen ejecutivo

No, todavía no probamos todas las alternativas con sentido.

Lo que sí hicimos hasta ahora:

- investigación documental y selección inicial por etapa
- prototipos Rust mínimos y compilables en `spikes/ras`
- validación básica de API y tests unitarios del workspace de spike

Lo que todavía falta si queremos decir que el barrido fue completo:

- benchmark real de fetch: `git` CLI vs `gix` vs `git2`
- benchmark real de walking: `ignore` vs `jwalk` vs `walkdir`
- comparación práctica de detección de lenguaje: `gengo` vs `tokei` vs `hyperpolyglot`
- validación de storage con carga real: `rusqlite` vs una opción KV sólo si SQLite mostrara ser cuello de botella

## Matriz por etapa

### 1. Fetch de repositorios

| Opción | Estado actual | ¿Tiene sentido probarla? | Veredicto actual |
| --- | --- | --- | --- |
| `git` CLI | investigada y elegida | sí | baseline obligatoria para V1 |
| `gix` | investigada y enlazada en prototipo | sí | principal alternativa Rust-native |
| `git2` / `libgit2` | investigada | sí | comparativa útil, no default |
| `jj` / Jujutsu | no evaluada a fondo | no | fuera de scope, no es backend Git drop-in para este caso |
| `hg` / otros VCS | no evaluados | no | fuera de scope V1 |

Conclusión:

- no falta ninguna alternativa principal de fetch dentro del scope GitHub/Git
- sí falta probar comparativamente `git` CLI, `gix` y `git2`

### 2. Lectura Git e historia

| Opción | Estado actual | ¿Tiene sentido probarla? | Veredicto actual |
| --- | --- | --- | --- |
| `gix` | investigada y recomendada | sí | favorita para lectura de objetos, refs e historia |
| `git2` | investigada | sí | fallback válido si aparece un gap concreto |
| `git` CLI + parsing | mencionada indirectamente | no, salvo fallback | útil como escape hatch, mala base para historia intensiva |

Conclusión:

- para historia Git, las alternativas que realmente importan son `gix` y `git2`
- no hace falta abrir más frentes aquí

### 3. Recorrido de working tree

| Opción | Estado actual | ¿Tiene sentido probarla? | Veredicto actual |
| --- | --- | --- | --- |
| `ignore` | investigada y usada en prototipo | sí | default por semántica `.gitignore` |
| `jwalk` | investigada | sí | benchmark de rendimiento obligatorio |
| `walkdir` | investigada | sí | baseline simple y madura |
| `globwalk` | investigada superficialmente | no como core | útil si el problema fuera glob-first, no repo scan general |
| `wax` | investigada superficialmente | no como core | potente, pero añade complejidad innecesaria para V1 |

Conclusión:

- sí conviene probar tres walkers: `ignore`, `jwalk`, `walkdir`
- `globwalk` y `wax` no son omisiones relevantes para este producto

### 4. Conteo de bytes, líneas y texto/binario

| Opción | Estado actual | ¿Tiene sentido probarla? | Veredicto actual |
| --- | --- | --- | --- |
| `memchr` + `bstr` | investigada y usada en prototipo | sí | mejor núcleo low-level |
| `content_inspector` | investigada y usada en prototipo | sí | complemento heurístico |
| `infer` | investigada y usada en prototipo | sí | complemento por firmas binarias |
| `tokei` | investigada | sí | buena referencia de contraste, no reemplazo completo |
| `rust-code-analysis` | investigada | no como core | demasiado pesado y orientado a métricas semánticas |

Conclusión:

- aquí no falta una alternativa core importante
- lo pendiente es medir precisión/coste, no descubrir nuevas librerías

### 5. Detección de lenguaje

| Opción | Estado actual | ¿Tiene sentido probarla? | Veredicto actual |
| --- | --- | --- | --- |
| `gengo` | investigada | sí | candidata principal, pero hay que validar con corpus propio |
| `tokei` | investigada | sí | baseline rápida para comparación |
| `hyperpolyglot` | investigada | sí | alternativa ligera que vale la pena medir |
| GitHub Linguist directo | evaluada indirectamente | no para core local | buena referencia conceptual, mala dependencia operativa local |
| `tree-sitter` | investigada | no para detección general | demasiado costosa como detector primario |

Conclusión:

- sí faltaba una alternativa con sentido en la matriz original: `hyperpolyglot`
- el set correcto para probar aquí es `gengo` vs `tokei` vs `hyperpolyglot`

### 6. Enriquecimiento semántico

| Opción | Estado actual | ¿Tiene sentido probarla? | Veredicto actual |
| --- | --- | --- | --- |
| `tree-sitter` | investigada | sí, más adelante | opción real para enriquecimiento opt-in |
| `rust-analyzer` | ya estaba en spec | sí, más adelante | sólo para etapa Rust específica y cacheada |
| `rust-code-analysis` | investigada | sí, secundaria | posible spike aparte, no núcleo V1 |

Conclusión:

- no hace falta probar esto ahora para decidir el core de V1
- sí conviene reservar un spike independiente después del pipeline base

### 7. Persistencia y checkpoints

| Opción | Estado actual | ¿Tiene sentido probarla? | Veredicto actual |
| --- | --- | --- | --- |
| `rusqlite` + SQLite | investigada y usada en prototipo | sí | baseline correcta para V1 |
| `sqlx` + SQLite | investigada | no de entrada | más pesada y menos alineada con CLI batch local |
| `diesel` + SQLite | investigada | no de entrada | demasiado ORM para este problema |
| `duckdb` | investigada | sí, pero para export/analytics | no para checkpoints operativos |
| `redb` | investigada | sí, sólo si SQLite falla | útil como KV puro, pierde modelo relacional natural |
| `sled` | investigada | no | alpha y menos convincente que `redb` hoy |
| `fjall` | investigada | no por ahora | interesante como KV, pero no mejora el caso central |
| `sea-orm` | investigada | no | capa async/ORM innecesaria para V1 |

Conclusión:

- la matriz original cubría la opción principal, pero faltaba explicitar alternativas modernas
- las únicas alternativas que merece la pena probar de verdad son `rusqlite` y, si hay problema real, `redb`
- `duckdb` tiene sentido para datasets derivados, no como motor de checkpoints

## Lista final de pruebas que sí faltan

Estas son las pruebas comparativas que todavía tienen sentido y no deberíamos omitir:

1. Fetch:
   `git` CLI vs `gix` vs `git2`
2. Walking:
   `ignore` vs `jwalk` vs `walkdir`
3. Detección de lenguaje:
   `gengo` vs `tokei` vs `hyperpolyglot`
4. Storage:
   `rusqlite` como baseline
   `redb` sólo si SQLite muestra dolor real en benchmarks o contención

## Alternativas que conscientemente no vale la pena perseguir ahora

- `jj` como backend principal
- ORMs completos como `diesel` o `sea-orm` para checkpoints
- motores analíticos como `duckdb` para la ruta operacional primaria
- `tree-sitter` como detector principal de lenguaje
- walkers orientados a globs como `globwalk` o `wax` como base del scan completo

## Fuentes

- [git clone](https://git-scm.com/docs/git-clone.html)
- [gix en docs.rs](https://docs.rs/gix)
- [git2 en docs.rs](https://docs.rs/git2)
- [ignore en docs.rs](https://docs.rs/ignore)
- [jwalk en crates.io](https://crates.io/crates/jwalk)
- [walkdir en crates.io](https://crates.io/crates/walkdir)
- [gengo en crates.io](https://crates.io/crates/gengo)
- [tokei en crates.io](https://crates.io/crates/tokei)
- [hyperpolyglot en crates.io](https://crates.io/crates/hyperpolyglot)
- [rusqlite en docs.rs](https://docs.rs/rusqlite)
- [redb en crates.io](https://crates.io/crates/redb)
- [duckdb en crates.io](https://crates.io/crates/duckdb)
