# ADR 0002: Backend para lectura Git e historia

Estado: Aceptado

Fecha: 2026-04-02

## Contexto

Después del fetch, `pulse` necesita leer:

- refs
- commits
- ranges
- árboles
- evolución semanal
- actividad de contribuyentes

Esta capa sí requiere semántica Git rica y acceso eficiente a objetos e historia.

## Decisión a tomar

Qué backend usar para lectura intensiva de repositorios Git y recorrido histórico.

## Alternativas consideradas

### Opción A: `gix`

Ventajas:

- Rust puro
- cobertura amplia de repos, objetos, refs, traversal y diff
- buen encaje con lectura intensiva local
- evita depender de parsing de comandos externos

Desventajas:

- curva de adopción mayor
- algunas áreas siguen evolucionando

### Opción B: `git2`

Ventajas:

- bindings maduros
- acceso programático directo a objetos Git
- alternativa conocida en el ecosistema Rust

Desventajas:

- FFI/libgit2
- peor alineación con la dirección Rust-first

### Opción C: `git` CLI + parsing

Ventajas:

- cero dependencia nueva
- fácil prototipado puntual

Desventajas:

- frágil para análisis intensivo
- parsing más costoso y menos tipado
- peor composición para recorridos complejos

## Decisión

Usar `gix` como backend principal de lectura Git e historia.

Alternativa propuesta si `gix` presentara un gap concreto:

- usar `git2` como fallback limitado a la pieza bloqueada

## Razonamiento

La fase de análisis histórico sí justifica inversión en una librería Rust-native. Aquí `gix` aporta valor real: tipos, acceso a objetos y recorrido sin depender de shell.

## Consecuencias

### Positivas

- mejor base para evolución semanal y contributor metrics
- más control sobre caching e invalidación
- alineación fuerte con la arquitectura objetivo

### Negativas

- más complejidad inicial que shelling out
- obliga a consolidar conocimiento de `gix`

## Validación recomendada

Construir un benchmark/harness sobre repos de prueba para medir:

- apertura de repo
- revwalk
- lectura de árboles
- extracción de commits por semana

Comparar:

1. `gix`
2. `git2`
