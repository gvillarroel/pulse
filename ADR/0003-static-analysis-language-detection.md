# ADR 0003: Análisis estático y detección de lenguaje

Estado: Aceptado

Fecha: 2026-04-02

## Contexto

La etapa de snapshot estático debe calcular:

- inventario de archivos
- conteo de bytes y líneas
- clasificación texto/binario
- lenguaje por archivo
- métricas agregadas por repositorio

Necesitamos una solución correcta, rápida y componible, no un stack semántico pesado.

## Decisiones a tomar

1. cómo recorrer el árbol
2. cómo contar bytes/líneas y clasificar binario
3. cómo detectar lenguaje

## Alternativas consideradas

### Walking del árbol

- `ignore`
- `jwalk`
- `walkdir`
- `globwalk`
- `wax`

### Conteo y clasificación

- `memchr` + `bstr`
- `content_inspector`
- `infer`
- `tokei`
- `rust-code-analysis`

### Detección de lenguaje

- `gengo`
- `tokei`
- `hyperpolyglot`
- `tree-sitter`

## Decisión

### Walking

Usar `ignore` como walker principal.

Alternativa propuesta:

- `jwalk` si el benchmark muestra mejora fuerte sin perder semántica necesaria

### Conteo/clasificación

Usar:

- `memchr` + `bstr` para conteo byte-oriented
- `content_inspector` + `infer` como complementos de clasificación

Alternativa propuesta:

- `tokei` como baseline comparativa, no como motor único del análisis

### Lenguaje

Usar `gengo` como candidato principal.

Alternativa propuesta:

- `tokei` o `hyperpolyglot` si `gengo` no resiste bien la validación sobre corpus real

## Razonamiento

- `ignore` resuelve correctamente la semántica `.gitignore`, que para este producto pesa más que exprimir el último porcentaje de velocidad
- `memchr` + `bstr` dan control fino y barato sobre bytes arbitrarios
- `gengo` parece el mejor encaje repo-level, pero todavía requiere validación real
- `tree-sitter` no debe entrar en el camino crítico de V1

## Consecuencias

### Positivas

- stack simple y controlable
- menor peso que una ruta semántica profunda
- separación clara entre snapshot base y enriquecimiento futuro

### Negativas

- exige benchmark explícito para walkers y detectores de lenguaje
- la decisión de lenguaje sigue provisional hasta medir precisión

## Validación recomendada

1. Walking:
   `ignore` vs `jwalk` vs `walkdir`
2. Lenguaje:
   `gengo` vs `tokei` vs `hyperpolyglot`
3. Exactitud:
   corpus propio con repos mixtos
