# 12 — DECISIONS

## 1. Propósito

Registro de **decisiones de fundación** (ADR ligero): contexto, decisión, alternativas, consecuencias.

Estado: `Accepted` | `Proposed` | `Superseded`.

---

## 2. Decisiones aceptadas

### D-001 — Producto independiente local

| | |
|--|--|
| **Estado** | Accepted |
| **Contexto** | Riesgo de nacer como módulo de VigilCut. |
| **Decisión** | Visual Library es producto desktop local independiente. |
| **Alternativas** | Monorepo compartido con acoplamiento; servicio cloud. |
| **Consecuencias** | Sin imports VigilCut; integración solo como consumer futuro. |

---

### D-002 — Organización por flujos, no por tablas

| | |
|--|--|
| **Estado** | Accepted |
| **Contexto** | Apps de assets degeneran en CRUD. |
| **Decisión** | Navegación = 6 flujos MVP. Entidades son internas. |
| **Consecuencias** | Estructura UI/application por factory/review/… |

---

### D-003 — Cadena Concept → Representation → Asset → Usage

| | |
|--|--|
| **Estado** | Accepted |
| **Contexto** | Evitar centrar el modelo en archivos de video/imagen. |
| **Decisión** | El concepto es el ancla semántica. |
| **Consecuencias** | Coverage y Plans razonan en conceptos/representaciones. |

---

### D-004 — Library gate (Review obligatorio)

| | |
|--|--|
| **Estado** | Accepted |
| **Contexto** | Generación sin curación contamina el catálogo. |
| **Decisión** | Todo generado entra `waiting_review`; solo `approved` en Library. |
| **Consecuencias** | Invariantes en dominio + queries Library. |

---

### D-005 — Separación Plans / Factory

| | |
|--|--|
| **Estado** | Accepted |
| **Contexto** | Mezclar “qué” y “cómo” produce generación caótica. |
| **Decisión** | Plans decide qué; Factory cómo. Automatic solo con plan approved. |
| **Consecuencias** | Use cases y UI separados; tests de gate. |

---

### D-006 — Stack Tauri 2 + Rust + SQLite + FS

| | |
|--|--|
| **Estado** | Accepted |
| **Contexto** | App local, performance, control FS, un solo binario. |
| **Decisión** | Tauri 2, core Rust, SQLite, filesystem administrado. |
| **Alternativas** | Electron; Postgres; cloud Supabase. |
| **Consecuencias** | Sin backend cloud en núcleo; migraciones SQLite propias. |

---

### D-007 — Jobs durables en SQLite

| | |
|--|--|
| **Estado** | Accepted |
| **Contexto** | Generación y batches no pueden vivir solo en RAM. |
| **Decisión** | Tabla jobs + worker en proceso + recovery al boot. |
| **Consecuencias** | Fase temprana de jobs antes de Factory real. |

---

### D-008 — `waiting_review` es del Asset, no del Job de generate

| | |
|--|--|
| **Estado** | **Superseded by D-019** |
| **Contexto** | Lista de estados de job incluye waiting_review. |
| **Decisión original** | Job `generate_asset` → `completed` al crear Asset en waiting_review. |
| **Reemplazo** | D-019: jobs de generación terminan en **`waiting_review`**, no `completed`. |

---

### D-009 — Provider stub antes que IA real

| | |
|--|--|
| **Estado** | Accepted |
| **Contexto** | No implementar IA en fundación/MVP de arquitectura. |
| **Decisión** | `StubImageProvider` desbloquea pipelines. |
| **Consecuencias** | MVP demostrable offline; adapter real post-MVP o fase P0+. |

---

### D-010 — Un solo worker de jobs en MVP

| | |
|--|--|
| **Estado** | Accepted |
| **Contexto** | SQLite y simplicidad de recovery. |
| **Decisión** | Concurrencia default 1; configurable después con cuidado. |
| **Consecuencias** | Menos throughput; más predicción. |

---

### D-011 — Concept ↔ Theme N:M

| | |
|--|--|
| **Estado** | Accepted |
| **Contexto** | Un concepto puede vivir en varios ejes temáticos. |
| **Decisión** | Tabla `concept_themes`. |
| **Alternativas** | 1:N theme_id en concepts. |
| **Consecuencias** | Queries de coverage por theme con join. |

---

### D-012 — IDs ULID string + timestamps ISO-8601 UTC TEXT

| | |
|--|--|
| **Estado** | Accepted |
| **Contexto** | Portabilidad SQLite y orden temporal legible. |
| **Decisión** | ULID + TEXT ISO-8601 UTC. |
| **Alternativas** | UUID v4; integer epoch. |
| **Consecuencias** | Consistencia en repos y tests. |

---

### D-013 — Matching FOUND MVP = exacto + approved

| | |
|--|--|
| **Estado** | Accepted |
| **Contexto** | Evitar ranking ML prematuro. |
| **Decisión** | Match por concept/representation + orientation/style (any permitido) + exclusion; ranking por `approved_at` desc. |
| **Consecuencias** | Predecible; extensible en Settings después. |

---

### D-014 — Regenerate supersedea el asset actual

| | |
|--|--|
| **Estado** | Accepted |
| **Contexto** | No mutar en silencio un binario en review. |
| **Decisión** | Asset actual → `superseded` (o rejected con razón regenerated); nuevo Asset `waiting_review`. |
| **Consecuencias** | Estado `superseded` en enum de Asset. |

---

### D-015 — Frontend TypeScript; React como default propuesto

| | |
|--|--|
| **Estado** | Accepted (framework default) |
| **Contexto** | UI en WebView Tauri. |
| **Decisión** | TypeScript strict; **React** por defecto en Fase 1 salvo objeción. |
| **Alternativas** | Svelte, Solid, Vue. |
| **Consecuencias** | Scaffold Fase 1 usa React; se puede cambiar solo en Fase 1. |

---

### D-016 — SQLite access con rusqlite + SQL migrations

| | |
|--|--|
| **Estado** | Accepted |
| **Contexto** | Desktop embebido, control de SQL. |
| **Decisión** | `rusqlite` + archivos `.sql` de migración. |
| **Alternativas** | sqlx; Diesel. |
| **Consecuencias** | SQL explícito; tests de migración. |

---

### D-017 — OS prioritario de desarrollo: Windows

| | |
|--|--|
| **Estado** | Accepted |
| **Contexto** | Entorno actual del repositorio. |
| **Decisión** | Primero Windows; diseño portable; CI multi-OS después. |
| **Consecuencias** | Paths y scripts validados en Windows primero. |

---

### D-018 — Entregas documentales sin commit automático

| | |
|--|--|
| **Estado** | Accepted |
| **Contexto** | Fundación y constituciones. |
| **Decisión** | No commit/push automáticos salvo autorización explícita. |
| **Consecuencias** | El usuario controla el historial git. |

---

### D-019 — Jobs de generación terminan en `waiting_review`

| | |
|--|--|
| **Estado** | Accepted |
| **Contexto** | Aprobación humana 2026-03-31 (sesión de constituciones). |
| **Decisión** | Jobs de **generación** terminan en **`waiting_review`**, **no** `completed`. Approve/Reject son transiciones posteriores sobre el Asset. |
| **Alternativas** | Job `completed` + flag `awaiting_human` (rechazada). |
| **Consecuencias** | Actualiza constitución Jobs; supersede D-008; UI y tests deben asertar `waiting_review`. |

---

### D-020 — Infra de tests FE: Vitest + Testing Library desde el inicio

| | |
|--|--|
| **Estado** | Accepted |
| **Decisión** | Instalar y mantener Vitest + Testing Library **antes** de implementar pantallas de negocio. |
| **Consecuencias** | `pnpm test` / scripts en `@visual-library/ui`; feature FE sin tests no es Done. |

---

### D-021 — E2E con Playwright sobre Vite

| | |
|--|--|
| **Estado** | Accepted |
| **Decisión** | E2E con **Playwright** contra el dev server **Vite**. No bloquear por integración completa Tauri. Pruebas específicas Tauri se añaden **después** cuando hagan falta. |
| **Consecuencias** | Suite E2E en repo; sin provider real; sin OmniRoute. |

---

### D-022 — Duplicados MVP = solo SHA-256 (sin pHash)

| | |
|--|--|
| **Estado** | Accepted |
| **Decisión** | MVP implementa **únicamente SHA-256** para duplicados exactos. **Perceptual hash (pHash) es post-MVP** (fase documentada, no mezclar en MVP). |
| **Consecuencias** | Data constitution y non-goals alineados; no código pHash en MVP. |

---

### D-023 — Secrets de providers en almacenamiento seguro del sistema

| | |
|--|--|
| **Estado** | Accepted |
| **Decisión** | Desde el **primer proveedor real**, API keys en **almacenamiento seguro del OS** (keychain/credential store). **Nunca** en SQLite, JSON, config en texto plano, ni logs. |
| **Consecuencias** | Security constitution; stub no requiere secrets. |

---

### D-024 — Calidad Rust: fmt + check obligatorios; clippy en CI

| | |
|--|--|
| **Estado** | Accepted |
| **Decisión** | Desde ahora: **`cargo fmt`** y **`cargo check`** obligatorios en tareas que toquen Rust. Cuando exista CI estable: **`cargo clippy` con `-D warnings`** como gate obligatorio. |
| **Consecuencias** | Scripts en package.json raíz; DoD/Testing constitutions actualizadas. |

---

### D-025 — SQLite: WAL, FKs, migraciones inmutables publicadas

| | |
|--|--|
| **Estado** | Accepted |
| **Decisión** | Activar **WAL** desde el inicio. **Foreign keys** activas. Migraciones **numeradas**. **Nunca** modificar migraciones ya publicadas (solo migraciones nuevas). |
| **Consecuencias** | Data constitution; aplica al implementar SQLite (negocio aún en pausa). |

---

### D-026 — Implementación de producto en pausa; solo infra + docs

| | |
|--|--|
| **Estado** | Accepted |
| **Decisión** | No implementar funcionalidades de negocio hasta nueva orden. Sí: constituciones, ADR, infraestructura de tests/calidad. |
| **Consecuencias** | Scaffold UI permanece placeholder; sin Factory/Review reales. |

---

### D-027 — Foundation 0 aprobada

| | |
|--|--|
| **Estado** | Accepted |
| **Contexto** | Aprobación humana explícita: “foundation 0 aprobada”. |
| **Decisión** | Se considera **cerrada y aprobada** la Foundation 0: AI_PLAYBOOK, PRODUCT, ARCHITECTURE, constituciones F0 (ENGINEERING, UX_UI, TESTING, SECURITY, DONE), START-HERE, FOUNDATION_REVIEW, Audit 01. |
| **Consecuencias** | Es la metodología oficial de trabajo. Foundation 1+ (p.ej. SQLite/settings) **no** arranca sola: requiere orden explícita. Legado 00–09 y 01/03 siguen no normativos. |

---

### D-028 — Foundation 1: plataforma de datos local

| | |
|--|--|
| **Estado** | Accepted |
| **Contexto** | Orden “avanza” tras F0 aprobada. |
| **Decisión** | Implementar **Foundation 1**: layout app-data, SQLite con **WAL + foreign_keys**, migraciones numeradas (`0001_init`), tabla `settings`, commands `get_app_paths` / `get_settings` / `set_media_root`. **Sin** tablas de catálogo ni Factory/Review. |
| **Consecuencias** | Settings persiste `media_root`; bootstrap en arranque Tauri. |

---

### D-029 — Foundation 2: dominio + catálogo base

| | |
|--|--|
| **Estado** | Accepted |
| **Contexto** | Orden “avanza” tras F1. |
| **Decisión** | Domain: `AssetStatus` transitions + `CoveragePlan` automatic gate. Persistencia: migración **`0002_domain_tables`**. Application: ensure/list Theme/Concept/Representation. API + Library UI mínima de ensure concept. **Sin** Factory generate, **sin** Review approve, **sin** job worker. |
| **Consecuencias** | Catálogo semántico listo para Factory/Review en fases siguientes. |

---

### D-030 — Foundation 3: generate stub + Review + Library assets

| | |
|--|--|
| **Estado** | Accepted |
| **Contexto** | Orden “avanza” tras F2. |
| **Decisión** | Job durable `generate_asset` (in-process) termina en **`waiting_review`**. Stub PNG + SHA-256. Review: list/approve/reject. Library lista solo **approved**. Idempotency key en jobs. |
| **Consecuencias** | Camino feliz generate → review → library sin Manual Factory completo aún. |

---

### D-031 — Foundation 4: Manual Factory FOUND/GENERATE

| | |
|--|--|
| **Estado** | Accepted |
| **Contexto** | Orden “avanza” tras F3. |
| **Decisión** | Manual Factory: lista de necesidades → **preview** FOUND/GENERATE (matching approved + orientation/style) → **submit** genera solo faltantes a Waiting Review. Auto-ensure concept/representation por keys. |
| **Consecuencias** | FOUND no crea asset; GENERATE reutiliza stub pipeline. Automatic Factory sigue pendiente. |

---

### D-032 — Foundation 5: Plans + Automatic Factory

| | |
|--|--|
| **Estado** | Accepted |
| **Contexto** | Orden “avanza” tras F4. |
| **Decisión** | Plans: create draft, add items, **approve** (sin generar). Automatic Factory: solo plan **approved**; items → FOUND/GENERATE; generate → waiting_review; items found→fulfilled, generate→scheduled. |
| **Consecuencias** | Plans ≠ Factory enforced. Draft no puede run automatic. |

---

### D-033 — Foundation 6: Coverage + Review completo

| | |
|--|--|
| **Estado** | Accepted |
| **Contexto** | Orden “avanza y cuando tengamos interfaz revisamos”. |
| **Decisión** | Coverage report con summary + issues accionables (CTA a flujos). Review MVP completo: Approve, Reject, Edit metadata, Regenerate (supersede + stub), Mark duplicate. |
| **Consecuencias** | Las 6 estaciones tienen función real de producto (salvo polish). |

---

## 3. Decisiones propuestas (abiertas)

### P-001 — Auto-ensure de Concept/Representation en Manual Factory

| | |
|--|--|
| **Estado** | Proposed |
| **Pregunta** | Si la lista trae un concept key nuevo, ¿se crea draft automáticamente o se falla la fila? |
| **Opciones** | (A) Auto-ensure draft · (B) Fail row · (C) Fail batch |
| **Recomendación** | **A** con flag visible en preview |

### P-003 — Bulk approve en Review MVP

| | |
|--|--|
| **Estado** | Proposed |
| **Pregunta** | ¿Permitir approve múltiple en MVP? |
| **Recomendación** | Sí, solo con selección explícita + confirmación |

### P-004 — Cache materializada de Coverage issues

| | |
|--|--|
| **Estado** | Proposed |
| **Pregunta** | ¿Issues always on-the-fly o cache + job? |
| **Recomendación** | On-the-fly hasta que duela |

### P-005 — Nombre de paquete/crate

| | |
|--|--|
| **Estado** | Proposed |
| **Recomendación** | `visual_library` en crates; repo `VisuaLibrary` |

---

## 4. Cómo añadir una decisión

1. Nuevo ID `D-xxx` o `P-xxx`.  
2. Contexto / decisión / alternativas / consecuencias.  
3. Si cambia arquitectura o dominio, actualizar el doc afectado.  
4. No silenciar cambios en código sin ADR.

---

## 5. Referencias

- Non-goals: [13-NON_GOALS.md](./13-NON_GOALS.md)  
- Architecture: [03-ARCHITECTURE.md](./03-ARCHITECTURE.md)  
- Domain: [02-DOMAIN.md](./02-DOMAIN.md)
