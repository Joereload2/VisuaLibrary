# PRODUCT — Visual Library

**Autoridad de producto (Foundation 0).**
`docs/01-PRODUCT.md` es referencia ampliada; si hay conflicto, **gana este archivo**.

---

## Visión

**Visual Library** es una aplicación de escritorio **local** para crear, organizar, revisar y consultar **recursos visuales reutilizables** centrados en **conceptos**, no en archivos sueltos.

Es un **producto independiente**. No es un módulo de VigilCut.
VigilCut podrá consumir; Visual Library **nunca depende** de VigilCut.

---

## Objetivos

1. Modelo conceptual explícito: Concepto → Representaciones → Assets → Uso.
2. Generación controlada (Factory) con reutilización (FOUND) antes de generar.
3. Gate de calidad humano (Review) antes del catálogo.
4. Biblioteca confiable (Library) solo con recursos aprobados.
5. Cobertura medible y accionable (Coverage).
6. Crecimiento planificado (Plans) separado de la ejecución (Factory).
7. Operable por **una sola persona** en máquina local.

---

## Filosofía de producto (no técnica)

| Principio | Significado |
|-----------|-------------|
| Conceptos primero | No girar alrededor de videos o carpetas de imágenes |
| Calidad > volumen | Nada generado entra a Library sin Approve |
| Plan ≠ Factory | Plans decide **qué**; Factory **cómo** |
| Flujos, no tablas | El usuario trabaja en estaciones, no en CRUD de entidades |
| Local-first | Datos en el equipo del usuario |
| Automatizado y supervisable | Jobs y generación automáticos; humanos revisan |

---

## MVP — seis flujos (solo estos)

| Flujo | Propósito |
|-------|-----------|
| **Factory** | Crear: Manual (lista de necesidades) y Automatic (desde plan aprobado). Salida: Waiting Review. |
| **Review** | Approve, Reject, Edit metadata, Regenerate, Mark duplicate. |
| **Library** | Buscar, filtrar, consultar, exportar **solo approved**. |
| **Coverage** | Problemas accionables de cobertura (no solo gráficos). |
| **Plans** | Definir qué crecer; approve habilita Automatic Factory. |
| **Settings** | Configuración local; sin producción. |

### Cadena feliz

```text
(Settings) → Plans y/o Manual Factory → Factory → Review → Library
                              ↑                         ↓
                           Coverage ←──────────────────┘
```

### Reglas de producto inquebrantables

- Generate → **Waiting Review** (nunca directo a Library).
- Automatic Factory **solo** con Coverage Plan **approved**.
- No generación aleatoria.
- No séptimo flujo primario en el MVP.

---

## Alcance del MVP (sí)

- App desktop local (Tauri).
- Dominio conceptual + persistencia local (SQLite + FS).
- Jobs durables.
- Provider de generación **stub** primero; provider real en fase posterior aprobada.
- Duplicados exactos por **SHA-256**.
- Export de información desde Library.
- Un usuario local (sin multi-usuario cloud).

---

## No objetivos (MVP y núcleo)

| No objetivo |
|-------------|
| Módulo de VigilCut o dependencia de VigilCut |
| Editor de video / NLE |
| DAM enterprise genérico |
| SaaS, Supabase, Postgres cloud, sync multi-dispositivo |
| OmniRoute / agentes / IA multi-paso en el núcleo |
| Navegación primaria “Conceptos / Assets / Representaciones” |
| Generar o aprobar desde Library |
| Generar desde Plans o Coverage |
| pHash / dedup perceptual (post-MVP documentado) |
| Telemetría cloud obligatoria |
| Marketplace / red social de assets |

Detalle ampliable: `docs/13-NON_GOALS.md`.

---

## Roles de usuario (una persona puede ser todos)

| Rol | Necesidad |
|-----|-----------|
| Productor | Manual Factory |
| Curador | Review |
| Bibliotecario | Library |
| Planificador | Coverage + Plans |
| Admin local | Settings |

---

## Métricas de éxito del MVP (producto)

| Métrica | Idea |
|---------|------|
| Zero-leak to Library | 0 assets en Library sin Approve |
| Reuse rate | FOUND vs GENERATE en Manual |
| Review lag | Cola waiting_review |
| Plan fidelity | Automatic solo ejecuta planes approved |
| Coverage actionability | Issues con siguiente acción clara |

---

## Fuera de este documento

Arquitectura, jobs, SQLite, UI técnica → `ARCHITECTURE.md` y constituciones.
