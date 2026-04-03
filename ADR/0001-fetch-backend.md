# ADR 0001: Backend de fetch de repositorios

Estado: Aceptado

Fecha: 2026-04-02

## Contexto

`pulse` necesita una etapa de fetch resumible para muchos repositorios:

- clone inicial
- actualización incremental
- cache local reutilizable bajo `--state-dir`
- recuperación tras fallos parciales
- compatibilidad robusta con autenticación y transporte Git reales

Esta etapa no necesita semántica rica de análisis. Necesita fiabilidad operativa.

## Decisión a tomar

Qué backend usar para clone/fetch en V1.

## Alternativas consideradas

### Opción A: `git` CLI

Ventajas:

- máxima compatibilidad real con protocolos y auth
- soporte natural para `clone --mirror`, `fetch`, `remote update`
- semántica operativa conocida y madura
- bordes de fallo claros vía subprocess

Desventajas:

- overhead de procesos
- parsing de stderr/stdout
- menos integración Rust-native

### Opción B: `gix`

Ventajas:

- Rust puro
- mayor control programático
- alineación estratégica a largo plazo

Desventajas:

- API más baja nivel
- mayor coste de implementación para cleanup/retry
- más riesgo de edge cases operativos en la primera versión

### Opción C: `git2` / `libgit2`

Ventajas:

- bindings maduros
- clone/fetch bien cubiertos
- integración directa desde Rust

Desventajas:

- dependencia nativa C/FFI
- peor encaje con el objetivo de distribución simple
- añade otra superficie operativa aparte de Git CLI y `gix`

## Decisión

Usar `gix` como backend de fetch para V1.

Diseño propuesto:

- cache bare o mirror persistente bajo `--state-dir/repos/...`
- checkpoint por repositorio y etapa
- metadata de revisión, timestamp y config hash
- interfaz Rust desacoplada para permitir fallback futuro

Alternativa descartada para V1:

- `git` CLI como baseline operativa de menor riesgo

## Razonamiento

Se prioriza alineación Rust-native desde el día uno, aun aceptando mayor coste inicial. La decisión favorece una arquitectura homogénea entre fetch e historia, reduce dependencia en procesos externos y evita fijar temprano un backend que luego haya que desmontar.

## Consecuencias

### Positivas

- arquitectura Git homogénea en Rust
- menos dependencia operativa en el binario `git`
- mejor continuidad entre fetch, lectura e historia

### Negativas

- mayor riesgo inicial en clone/fetch que con `git` CLI
- más trabajo de endurecimiento y benchmarking desde el principio
- puede aparecer algún gap operativo que obligue a fallback puntual

## Validación recomendada

Benchmark comparativo recomendado igualmente:

1. `gix`
2. `git` CLI
3. `git2`

Medir:

- clone fresco
- fetch incremental
- recuperación tras interrupción
- footprint de disco
- comportamiento ante errores
