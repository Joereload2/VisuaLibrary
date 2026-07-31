# 11 — IMPLEMENTATION PLAN

## 1. Propósito

Dividir **todo** el proyecto en **fases muy pequeñas**.
**Ninguna fase se implementa en esta entrega.** Solo se planifica.

Cada fase incluye:

- Objetivo
- Archivos (esperados)
- Dependencias
- Pruebas
- Riesgos
- Criterio de terminado

---

## 2. Reglas del plan

1. Una fase = un incremento revisable (idealmente horas–pocos días, no semanas monstruo).
2. No IA real ni OmniRoute en fases del núcleo.
3. Provider **stub** hasta fase explícita de providers.
4. No abrir fases de UI rica antes del shell y del backend mínimo.
5. No “implementar Factory completo” en una sola fase.
6. Cada fase debe poder demostrarse sin la siguiente.
7. **Fase 1 solo tras aprobación humana de esta documentación.**

---

## 3. Mapa de fases (visión)

| Grupo | Fases | Tema |
|-------|-------|------|
| Fundación repo | 0–1 | Docs gate, monorepo shell |
| Plataforma | 2–5 | SQLite, FS, jobs, settings |
| Dominio base | 6–8 | Entities + policies + tests |
| Read models | 9–11 | Library/Coverage/Plans lectura |
| Curation | 12–14 | Review write path |
| Acquisition | 15–19 | Factory manual/auto + generate stub |
| UX cierre MVP | 20–22 | Flujos UI mínimos + E2E |
| Hardening | 23–24 | Recovery, export, polish gates |

Los números son orden lógico; pueden re-etiquetarse al ejecutar, no saltarse dependencias.

---

## 4. Fases detalladas

### Fase 0 — Gate de fundación documental

| | |
|--|--|
| **Objetivo** | Documentación aprobada; equipo alineado. |
| **Archivos** | `docs/**` (este paquete) |
| **Dependencias** | Ninguna |
| **Pruebas** | Revisión humana de checklist en 00-START-HERE |
| **Riesgos** | Scope creep antes de scaffold |
| **Criterio de terminado** | Aprobación explícita: “Proceder con Fase 1” |

---

### Fase 1 — Scaffold monorepo (sin negocio)

| | |
|--|--|
| **Objetivo** | Repo compilable: Cargo workspace + Tauri hello + UI shell vacío. |
| **Archivos** | `Cargo.toml`, `apps/desktop/**`, `crates/domain` (lib vacía), `crates/application`, `crates/infrastructure`, `packages/ui/**`, `.gitignore`, `README.md` actualizado |
| **Dependencias** | Fase 0 |
| **Pruebas** | `cargo build`; app abre ventana; test `domain` trivial |
| **Riesgos** | Pelearse con toolchain Tauri en Windows |
| **Criterio de terminado** | Ventana con título Visual Library; 6 rutas placeholder; cero lógica de catálogo |

---

### Fase 2 — App data paths + Settings storage mínimo

| | |
|--|--|
| **Objetivo** | Resolver app_data_dir; leer/escribir settings key-value. |
| **Archivos** | `infrastructure/config`, `settings` repo, commands `settings_*`, UI Settings form mínimo (path display) |
| **Dependencias** | Fase 1 |
| **Pruebas** | unit settings merge; integration temp config |
| **Riesgos** | Paths Windows vs portable |
| **Criterio de terminado** | Reiniciar app conserva un setting de prueba |

---

### Fase 3 — SQLite open + migraciones framework

| | |
|--|--|
| **Objetivo** | Abrir DB, aplicar migraciones versionadas, tabla `schema_migrations`. |
| **Archivos** | `infrastructure/sqlite/**`, `migrations/0001_init.sql` (puede ser solo migrations + settings table) |
| **Dependencias** | Fase 2 |
| **Pruebas** | migrate on empty; second open no re-aplica |
| **Riesgos** | WAL + path locks en Windows |
| **Criterio de terminado** | DB creada bajo app data; test verde |

---

### Fase 4 — Migración de esquema de dominio (tablas MVP)

| | |
|--|--|
| **Objetivo** | Crear tablas de 05-DATABASE (sin lógica de uso completa). |
| **Archivos** | `migrations/0002_domain_tables.sql` (o expandir 0001 si aún no hay prod data) |
| **Dependencias** | Fase 3 |
| **Pruebas** | migrate; pragma foreign_keys; insert smoke por tabla crítica |
| **Riesgos** | Sobre-modelar; mitigar pegándose al doc 05 |
| **Criterio de terminado** | Esquema aplica limpio; índices clave existen |

---

### Fase 5 — MediaStore FS + path safety

| | |
|--|--|
| **Objetivo** | Media root, allocate path, write atomic, reject traversal. |
| **Archivos** | `infrastructure/fs_media/**`, settings media_root, tests |
| **Dependencias** | Fase 2–3 |
| **Pruebas** | fs tests de 10-QA §7 |
| **Riesgos** | Permisos antivirus Windows |
| **Criterio de terminado** | Escribir/leer fixture bajo root; `../` falla |

---

### Fase 6 — Jobs table + worker echo + recovery

| | |
|--|--|
| **Objetivo** | Cola durable, claim, complete/fail, recovery interrupted. |
| **Archivos** | `jobs` repo, worker, handler `echo`, command list/cancel, boot recovery |
| **Dependencias** | Fase 3–4 |
| **Pruebas** | jobs tests 10-QA §8 (echo + recovery) |
| **Riesgos** | Deadlocks SQLite; mitigar 1 worker |
| **Criterio de terminado** | Echo job sobrevive restart simulado; estados en DB |

---

### Fase 7 — Domain: Asset status machine + tests

| | |
|--|--|
| **Objetivo** | Modelo Asset + transiciones approve/reject/duplicate/supersede. |
| **Archivos** | `crates/domain/src/asset/**` |
| **Dependencias** | Fase 1 |
| **Pruebas** | unit transitions legales/ilegales |
| **Riesgos** | Estados extra no documentados |
| **Criterio de terminado** | Invariante Library gate expresable en dominio |

---

### Fase 8 — Domain: Plan gate + Found policy + Exclusion

| | |
|--|--|
| **Objetivo** | Políticas puras de plan approved y FOUND/GENERATE. |
| **Archivos** | `domain/plan`, `domain/generation/found_policy`, `domain/exclusion` |
| **Dependencias** | Fase 7 |
| **Pruebas** | unit policy matrix |
| **Riesgos** | Matching demasiado “smart” prematuro |
| **Criterio de terminado** | Tests de matching exacto MVP verdes |

---

### Fase 9 — Repos SQLite: Concepts, Representations, Themes

| | |
|--|--|
| **Objetivo** | Persistencia y lecturas básicas de catálogo. |
| **Archivos** | repos + application list/get mínimos + seed fixture helper |
| **Dependencias** | Fase 4, 8 |
| **Pruebas** | sqlite insert/list; unique key constraints |
| **Riesgos** | Exponer CRUD UI de conceptos; **no hacerlo** |
| **Criterio de terminado** | Seed programático usable por tests; sin página Conceptos |

---

### Fase 10 — Library search (read-only approved)

| | |
|--|--|
| **Objetivo** | Caso de uso + command + UI Library lista vacía/con seed. |
| **Archivos** | `application/library/search`, asset repo queries, UI library flow |
| **Dependencias** | Fase 9, 7 |
| **Pruebas** | waiting no aparece; approved sí |
| **Riesgos** | Filtros scope creep |
| **Criterio de terminado** | Search solo approved con fixture |

---

### Fase 11 — Plans draft CRUD + approve (sin execute)

| | |
|--|--|
| **Objetivo** | Crear plan/items; aprobar; no generar. |
| **Archivos** | plans use cases, commands, UI plans mínima |
| **Dependencias** | Fase 9 |
| **Pruebas** | approve cambia status; draft no “can_run” |
| **Riesgos** | Botón generar prematuro en UI |
| **Criterio de terminado** | Plan approved en DB; cero assets nuevos |

---

### Fase 12 — Review list waiting + approve/reject

| | |
|--|--|
| **Objetivo** | Cola real y dos acciones primarias. |
| **Archivos** | review use cases, commands, UI review |
| **Dependencias** | Fase 7, 10 (assets existen) |
| **Pruebas** | approve → library; reject → no library |
| **Riesgos** | Bulk inseguro |
| **Criterio de terminado** | E2E-05/06 a nivel application+UI mínima |

---

### Fase 13 — Review: edit metadata + mark duplicate

| | |
|--|--|
| **Objetivo** | Completar acciones de metadata y duplicate. |
| **Archivos** | use cases + UI actions |
| **Dependencias** | Fase 12 |
| **Pruebas** | duplicate requiere target; metadata no cambia status |
| **Riesgos** | Duplicate sin link |
| **Criterio de terminado** | 4/5 acciones Review (falta regen) |

---

### Fase 14 — Review: regenerate (request + job stub)

| | |
|--|--|
| **Objetivo** | Regenerate encola generate; supersede policy. |
| **Archivos** | regenerate use case, job generate stub, domain superseded |
| **Dependencias** | Fase 6, 12, 15 parcial (puede unificarse con generate) |
| **Pruebas** | nuevo asset waiting; viejo no approved auto |
| **Riesgos** | Doble generación; idempotencia |
| **Criterio de terminado** | Acción regenerate end-to-end con stub |

---

### Fase 15 — GenerationRequest + Stub ImageProvider + generate_asset job

| | |
|--|--|
| **Objetivo** | Pipeline de generación local falsa: bytes stub → FS → asset waiting_review. |
| **Archivos** | provider stub, job handler, media write, request repo |
| **Dependencias** | Fase 5, 6, 7, 9 |
| **Pruebas** | job **waiting_review** + asset waiting_review; no approved; idempotencia request (D-019) |
| **Riesgos** | Acoplar a provider real demasiado pronto |
| **Criterio de terminado** | Un command de test/dev puede generar 1 asset stub a Review |

---

### Fase 16 — Manual Factory: preview FOUND/GENERATE

| | |
|--|--|
| **Objetivo** | Resolver lista estructurada sin generar. |
| **Archivos** | manual_preview use case, parser lista, UI manual factory |
| **Dependencias** | Fase 8, 9, 10 |
| **Pruebas** | fixture needs → decisions correctas |
| **Riesgos** | Auto-crear concepts sin control |
| **Criterio de terminado** | Preview muestra FOUND/GENERATE estable |

---

### Fase 17 — Manual Factory: submit generate faltantes

| | |
|--|--|
| **Objetivo** | Encolar generate solo GENERATE; batch summary. |
| **Archivos** | manual_submit, batch entity, UI submit |
| **Dependencias** | Fase 15, 16 |
| **Pruebas** | FOUND no encola; GENERATE encola N jobs |
| **Riesgos** | Doble submit |
| **Criterio de terminado** | E2E-04 application+UI |

---

### Fase 18 — Automatic Factory: materialize plan

| | |
|--|--|
| **Objetivo** | De plan approved a GenerationRequests + jobs. |
| **Archivos** | automatic_run, materialize job/handler, UI automatic |
| **Dependencias** | Fase 11, 15, 16 (found policy) |
| **Pruebas** | draft falla; approved crea requests |
| **Riesgos** | Marcar items fulfilled demasiado pronto |
| **Criterio de terminado** | E2E-07/08 sin provider real |

---

### Fase 19 — Plan item progress (scheduled/fulfilled)

| | |
|--|--|
| **Objetivo** | Actualizar items según assets approved y jobs. |
| **Archivos** | application policy progress, coverage hooks |
| **Dependencias** | Fase 12, 18 |
| **Pruebas** | generate no cumple; approve sí |
| **Riesgos** | Race conditions |
| **Criterio de terminado** | Progreso de plan coherente post-review |

---

### Fase 20 — Coverage summary + issues accionables

| | |
|--|--|
| **Objetivo** | Computar métricas e issues; UI Coverage. |
| **Archivos** | coverage use cases, queries, UI |
| **Dependencias** | Fase 9–12 |
| **Pruebas** | fixtures under/over; review_backlog issue |
| **Riesgos** | Dashboard vanidoso sin CTAs |
| **Criterio de terminado** | E2E-09; issues con codes documentados |

---

### Fase 21 — Library export info + AssetUsage

| | |
|--|--|
| **Objetivo** | Exportar manifest de approved; registrar usage opcional. |
| **Archivos** | export use case, job opcional, usage repo, UI export |
| **Dependencias** | Fase 10, 5 |
| **Pruebas** | rejected excluido; usage row creada |
| **Riesgos** | Copiar árboles enormes; empezar por manifest |
| **Criterio de terminado** | Manifest JSON válido en carpeta export |

---

### Fase 22 — UI shell polish mínimo + navegación badges

| | |
|--|--|
| **Objetivo** | Badges review count, empty states, deep links Coverage→Plans/Review. |
| **Archivos** | `packages/ui/app`, flows empty states |
| **Dependencias** | Fases de flujos existentes |
| **Pruebas** | E2E-01; checks de no-rutas-entidad |
| **Riesgos** | Scope de diseño visual |
| **Criterio de terminado** | UX acceptance 09 §12 sin pixel polish |

---

### Fase 23 — Hardening jobs + integridad FS-DB

| | |
|--|--|
| **Objetivo** | Heartbeat, reintentos, health issue missing file. |
| **Archivos** | worker improvements, coverage integrity issue |
| **Dependencias** | Fase 6, 15, 20 |
| **Pruebas** | max_attempts; missing file detection |
| **Riesgos** | Over-engineering |
| **Criterio de terminado** | Checklist jobs 10-QA completa |

---

### Fase 24 — MVP acceptance gate

| | |
|--|--|
| **Objetivo** | Correr batería de aceptación de los 6 flujos; documentar gaps. |
| **Archivos** | `docs` notes / test reports (no nuevas features) |
| **Dependencias** | Fases 1–23 relevantes |
| **Pruebas** | checklist 10-QA §10 + E2E canónicos |
| **Riesgos** | Presión por meter IA real |
| **Criterio de terminado** | MVP declarado con stub provider; lista explícita post-MVP |

---

## 5. Fases explícitamente posteriores (post-MVP / no planificar implementación ahora)

Estas **no** forman parte del MVP a construir aún:

- P0+: Provider real de imágenes (un adapter) + OS secure store para secrets (D-023)
- P1: Import de assets existentes
- P1: FTS search
- P1: Thumbnails pipeline
- P1: **Perceptual hash (pHash)** — MVP solo SHA-256 (D-022)
- P2: Integración consumidor VigilCut
- P2: Multi-provider routing (no OmniRoute completo)
- P2: E2E específicos de shell Tauri (base E2E = Playwright+Vite, D-021)
- PX: OmniRoute / agentes / cloud

---

## 6. Dependencias críticas (DAG resumido)

```
0 → 1 → 2 → 3 → 4 → 9 → 10 → 12 → 13
         ↘ 5 ↗         ↘ 11 → 18 → 19
         ↘ 6 ↗→ 15 → 16 → 17
                ↘ 14
1 → 7 → 8 ↗
9 → 20 → 22
10 → 21
6 → 23 → 24
```

---

## 7. Política de cambios al plan

- Si una fase crece > 1 PR grande, **partirla**.
- Si se descubre necesidad de IA real, nueva fase post-24, no “meter en 15”.
- Toda fase nueva debe actualizar este documento **antes** de codificar.

---

## 8. Qué se entrega al cerrar el MVP (Fase 24)

- App local Tauri con 6 estaciones.
- Manual + Automatic Factory con **stub**.
- Review completo de acciones MVP.
- Library search + export info.
- Coverage actionable.
- Plans approve → factory.
- Jobs durables.
- SQLite + FS.
- Tests de pirámide mínima en verde.

**Sin:** IA real, OmniRoute, dependencia VigilCut, nube.

---

## 9. Esperando

**No implementar ninguna fase hasta:**

> Aprobación explícita de la Fase 1.

---

## 10. Referencias

- QA: [10-QA.md](./10-QA.md)
- Architecture: [03-ARCHITECTURE.md](./03-ARCHITECTURE.md)
- Non-goals: [13-NON_GOALS.md](./13-NON_GOALS.md)
