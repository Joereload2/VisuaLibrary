# FOUNDATION REVIEW — Visual Library

**Fecha de revisión:** 2026-07-31
**Última auditoría:** Foundation Audit 01 (misma fecha)
**Alcance:** fundación documental + metodología + scaffold + decisiones hasta D-026
**Estado de negocio:** **en pausa** (sin funcionalidades de catálogo/Factory/Review reales)
**Foundation 0:** **APROBADA** (D-027)  
**Próximo hito:** Foundation 1 solo con orden explícita (no automática)

### Post–Audit 01 (calidad documental)

- Autoridad única: 8 docs Foundation 0 + ADR; legado 00–09 y 01/03 marcados no normativos.
- Contradicciones D-019 corregidas en `04-WORKFLOWS`, `05-DATABASE`, `06-JOBS`, `11-IMPLEMENTATION_PLAN`.
- Playbook simplificado: multi-rol solo HIGH/ARCHITECTURE; DoR ligero para LOW; STOP “≥3 capas” medible.
- Ver `constitution/README.md`.

Este documento es un **mapa único** de todo lo construido en la fundación.
No sustituye los docs normativos; los **resume y localiza**.

---

## 1. Propósito de la fundación

Se construyó la base para que:

1. El producto tenga **identidad y alcance** claros.
2. El dominio y la arquitectura existan **antes** del código de negocio.
3. Las IAs (y humanos) desarrollen con la **misma metodología**.
4. Los prompts futuros puedan ser **cortos** (inteligencia en el repo).
5. Haya **scaffold técnico** y **infra de tests**, sin lógica de catálogo.

---

## 2. Estructura creada

### 2.1 Repositorio / monorepo (código)

```text
VisuaLibrary/
  README.md
  package.json                 # scripts pnpm monorepo
  pnpm-workspace.yaml
  pnpm-lock.yaml
  Cargo.toml                   # workspace Rust
  Cargo.lock
  .gitignore

  apps/desktop/                # @visual-library/desktop
    package.json               # tauri dev/build
    src-tauri/
      Cargo.toml               # visual_library_desktop
      tauri.conf.json          # título "Visual Library", Vite :1420
      capabilities/default.json
      src/main.rs, lib.rs      # command health + shell
      build.rs
      icons/ …                 # generados
      app-icon.png

  packages/ui/                 # @visual-library/ui
    package.json
    vite.config.ts             # Vite + Vitest
    playwright.config.ts       # E2E sobre Vite (no Tauri completo)
    index.html
    e2e/shell.spec.ts          # smoke 6 estaciones
    src/
      main.tsx, app/App.tsx, styles.css
      flows/{factory,review,library,coverage,plans,settings}/
      shared/{StationPlaceholder,ipc/client}
      test/setup.ts
      shared/StationPlaceholder.test.tsx

  crates/
    domain/                    # visual_library_domain (scaffold + tests)
    application/               # visual_library_application
    infrastructure/            # visual_library_infrastructure

  docs/                        # ver §4
```

### 2.2 Qué hay implementado en código (y qué no)

| Implementado | No implementado (pausa de negocio) |
|--------------|-------------------------------------|
| Workspace Cargo + pnpm | Dominio de catálogo (Concept, Asset, …) |
| Tauri 2 shell + ventana | SQLite / migraciones en código |
| 6 rutas UI placeholder | Jobs worker / tabla jobs |
| Command `health` | Factory Manual/Automatic real |
| Vitest + 1 test de placeholder | Review / Library / Coverage / Plans reales |
| Playwright config + e2e shell | Providers de imagen reales |
| Scripts fmt/check/test | OmniRoute, VigilCut, cloud |
| | ESLint, Prettier, CI GitHub Actions |

### 2.3 Scripts reales (`package.json` raíz)

| Script | Acción |
|--------|--------|
| `pnpm dev` / `build` | Tauri desktop |
| `pnpm dev:ui` / `build:ui` | Vite UI |
| `pnpm test` | test:ui + test:rust |
| `pnpm test:ui` | Vitest |
| `pnpm test:e2e` | Playwright |
| `pnpm test:e2e:install` | browsers Chromium |
| `pnpm test:rust` | `cargo test --workspace` |
| `pnpm fmt:rust` / `fmt:rust:check` | `cargo fmt` |
| `pnpm check:rust` | `cargo check --workspace` |
| `pnpm quality:rust` | fmt + check + test Rust |
| `pnpm quality` | fmt check + rust test + ui test + build:ui |

---

## 3. Documentos

### 3.1 Foundation 0 — metodología oficial (prioridad máxima)

En conflicto con docs antiguos, **gana Foundation 0** + `12-DECISIONS.md`.

| Archivo | Rol |
|---------|-----|
| [AI_PLAYBOOK.md](./AI_PLAYBOOK.md) | Cómo trabaja una IA: DoR, STOP rules, riesgo, orden de capas, task card, revisión multi-rol, 2 loops, entrega |
| [PRODUCT.md](./PRODUCT.md) | Visión, MVP, 6 flujos, alcance / no-objetivos (sin arquitectura) |
| [ARCHITECTURE.md](./ARCHITECTURE.md) | Capas, monorepo, fronteras, contratos |
| [constitution/ENGINEERING.md](./constitution/ENGINEERING.md) | BE/FE técnico, SQLite, FS, jobs, API, errores |
| [constitution/UX_UI.md](./constitution/UX_UI.md) | Constitución visual desktop-first + checklist |
| [constitution/TESTING.md](./constitution/TESTING.md) | Unit→E2E según riesgo LOW/MEDIUM/HIGH/ARCHITECTURE |
| [constitution/SECURITY.md](./constitution/SECURITY.md) | Secrets OS-only, paths, logs, integridad catálogo |
| [constitution/DONE.md](./constitution/DONE.md) | Checklist de cierre (sin prosa) |
| [00-START-HERE.md](./00-START-HERE.md) | Índice que apunta a Foundation 0 |
| **Este archivo** | [FOUNDATION_REVIEW.md](./FOUNDATION_REVIEW.md) |

### 3.2 Fundación de diseño detallada (referencia profunda)

Creada al inicio del proyecto; sigue siendo útil como profundidad.
No es la “puerta de entrada” principal.

| Archivo | Contenido |
|---------|-----------|
| [01-PRODUCT.md](./01-PRODUCT.md) | Producto extendido |
| [02-DOMAIN.md](./02-DOMAIN.md) | Entidades, invariantes, agregados |
| [03-ARCHITECTURE.md](./03-ARCHITECTURE.md) | Arquitectura extendida / monorepo propuesto |
| [04-WORKFLOWS.md](./04-WORKFLOWS.md) | Seis flujos y transiciones |
| [05-DATABASE.md](./05-DATABASE.md) | Esquema SQLite lógico, índices, queries |
| [06-JOBS.md](./06-JOBS.md) | Jobs durables; **actualizado** generate → `waiting_review` |
| [07-FRONTEND.md](./07-FRONTEND.md) | Organización FE por flujos |
| [08-BACKEND.md](./08-BACKEND.md) | Crates, ports, use cases |
| [09-UX.md](./09-UX.md) | Estaciones (sin UI pixel) |
| [10-QA.md](./10-QA.md) | Estrategia QA extendida |
| [11-IMPLEMENTATION_PLAN.md](./11-IMPLEMENTATION_PLAN.md) | Fases 0–24 + post-MVP |
| [12-DECISIONS.md](./12-DECISIONS.md) | ADR D-001…D-026 + propuestas abiertas |
| [13-NON_GOALS.md](./13-NON_GOALS.md) | Anti-alcance |

### 3.3 Constituciones numeradas (serie 00–09)

Serie larga creada antes de Foundation 0; **puede solaparse**.
Usar Foundation 0 en el día a día; 00–09 como detalle si hace falta.

| Archivo | Tema |
|---------|------|
| `constitution/00-ENGINEERING-CONSTITUTION.md` | Ingeniería global |
| `constitution/01-UX-UI-CONSTITUTION.md` | UX/UI |
| `constitution/02-FRONTEND-CONSTITUTION.md` | Frontend |
| `constitution/03-BACKEND-CONSTITUTION.md` | Backend |
| `constitution/04-DATA-CONSTITUTION.md` | SQLite + FS |
| `constitution/05-JOBS-CONSTITUTION.md` | Jobs (**generate → waiting_review**) |
| `constitution/06-TESTING-CONSTITUTION.md` | Testing |
| `constitution/07-SECURITY-PRIVACY-CONSTITUTION.md` | Seguridad |
| `constitution/08-OBSERVABILITY-CONSTITUTION.md` | Logs / métricas locales |
| `constitution/09-DEFINITION-OF-DONE.md` | Definition of Done extendida |

### 3.4 Mapa mental de lectura

```text
00-START-HERE / AI_PLAYBOOK
    ├── PRODUCT
    ├── ARCHITECTURE
    └── constitution/{ENGINEERING, UX_UI, TESTING, SECURITY, DONE}
            │
            ├── (detalle) 01–13, constitution/00–09
            └── 12-DECISIONS (verdad de ADR)
```

---

## 4. Decisiones (ADR)

Fuente canónica: [12-DECISIONS.md](./12-DECISIONS.md).

### 4.1 Aceptadas (resumen)

| ID | Decisión |
|----|----------|
| **D-001** | Producto local independiente; no módulo VigilCut |
| **D-002** | Navegación por **6 flujos**, no por tablas/entidades |
| **D-003** | Cadena **Concept → Representation → Asset → Usage** |
| **D-004** | Library gate: solo **Approve** entra a Library |
| **D-005** | **Plans = qué**; **Factory = cómo** |
| **D-006** | Stack **Tauri 2 + Rust + SQLite + FS** |
| **D-007** | Jobs **durables** en SQLite (no solo memoria) |
| **D-008** | *Superseded by D-019* (histórico: job generate → completed) |
| **D-009** | Provider **stub** antes que IA real |
| **D-010** | Un worker de jobs; concurrency default 1 en MVP |
| **D-011** | Concept ↔ Theme **N:M** (`concept_themes`) |
| **D-012** | IDs **ULID** string; timestamps **ISO-8601 UTC TEXT** |
| **D-013** | FOUND MVP: match exacto + approved (+ any style/orientation) |
| **D-014** | Regenerate: asset actual **superseded**; nuevo en waiting_review |
| **D-015** | Frontend **TypeScript + React** |
| **D-016** | SQLite vía **rusqlite** + migraciones SQL |
| **D-017** | OS prioritario de dev: **Windows** (diseño portable) |
| **D-018** | No commit/push automáticos sin autorización |
| **D-019** | Jobs de **generación** terminan en **`waiting_review`**, no `completed` |
| **D-020** | **Vitest + Testing Library** desde el inicio |
| **D-021** | E2E **Playwright sobre Vite**; Tauri E2E después |
| **D-022** | Duplicados MVP = **solo SHA-256**; pHash post-MVP |
| **D-023** | Secrets de providers en **OS secure store**; nunca SQLite/JSON/plano/logs |
| **D-024** | **`cargo fmt` + `cargo check`** obligatorios; clippy `-D warnings` cuando CI estable |
| **D-025** | SQLite: **WAL** + **FKs ON** + migraciones numeradas **inmutables** una vez publicadas |
| **D-026** | Implementación de **producto en pausa**; solo infra/docs hasta nueva orden |

### 4.2 Propuestas aún abiertas (no bloquean Foundation 0)

| ID | Tema |
|----|------|
| **P-001** | Auto-ensure Concept/Representation en Manual Factory (¿crear draft o fallar fila?) |
| **P-003** | Bulk approve en Review MVP (sí con selección + confirmación vs no) |
| **P-004** | Coverage issues on-the-fly vs cache materializada |
| **P-005** | Nombre de crates (`visual_library` ya en uso; formalizar) |

### 4.3 Decisiones de sesión (constituciones / playbook)

Además de los ADR, quedaron fijadas en playbook/constituciones:

- Definition of Ready + STOP rules para IAs.
- Orden de implementación de features (Producto → … → UI → QA).
- Clasificación de riesgo LOW / MEDIUM / HIGH / ARCHITECTURE.
- Revisión multi-rol al cerrar (PM, PO, UX, FE, BE, QA, Arquitectura).
- Dos loops: (1) errores (2) simplificación sin rediseñar.
- Entrega sin commit/push por defecto.

---

## 5. Constituciones — qué rige qué

### 5.1 Serie Foundation 0 (operativa)

| Constitución | Cubre |
|--------------|--------|
| **ENGINEERING** | Capas, API use-case, SQLite/WAL/FK/migraciones, FS, jobs, FE técnico, providers, calidad Rust/TS |
| **UX_UI** | Desktop-first, 6 flujos, sin overflow, preview 70–75%, paneles ≤30%, una acción primaria, estados, a11y, checklist |
| **TESTING** | Unit / Integration / Smoke / E2E / Regression; matriz por riesgo; comandos reales |
| **SECURITY** | Local-first, keys solo OS store, paths, cleanup, logs, integridad catálogo, IPC Tauri |
| **DONE** | Checklist de cierre por área + bloqueos duros |

### 5.2 Serie 00–09 (detalle histórico)

Misma temática desglosada (FE/BE/Data/Jobs/Obs/DoD).
**Observability** está principalmente en `08-OBSERVABILITY-CONSTITUTION.md` (no hay archivo corto homónimo en F0; se puede consolidar más adelante).

### 5.3 AI_PLAYBOOK (meta-constitución de proceso)

Es la constitución de **proceso**:

- Cómo empezar (TASK CARD + DoR).
- Cuándo parar (STOP RULES).
- Cómo clasificar riesgo.
- En qué orden implementar capas.
- Cómo revisar y entregar.

---

## 6. Reglas principales (no negociables)

### 6.1 Producto

1. App **local-first**, operable por **una persona**.
2. Independiente de **VigilCut** (puede consumir; no al revés).
3. Exactamente **6 flujos**: Factory, Review, Library, Coverage, Plans, Settings.
4. Centro = **Concepto**, no el archivo de imagen/video.
5. **Plans ≠ Factory**.
6. Generate → **Waiting Review** → solo **Approve** → Library.
7. Automatic Factory solo con plan **approved**.
8. Sin OmniRoute / IA de negocio / providers reales hasta fase aprobada (stub primero).
9. MVP duplicados = **SHA-256** (no pHash).
10. Negocio en **pausa** hasta Foundation 1+ aprobada.

### 6.2 Arquitectura / ingeniería

1. Capas: `domain` (puro) → `application` → `infrastructure` / commands / UI.
2. UI y application por **flujos**, no por tablas.
3. SQLite = metadata/estados; FS = bytes.
4. No canonicidad paralela (memoria + FE store + JSON + SQLite).
5. Commands = **casos de uso**, no `set_status` / `run_sql`.
6. Errores estructurados: code, message, retryable, suggested_action, detail?.
7. Sin `unwrap`/`expect`/`panic` en rutas productivas.
8. Migraciones numeradas; **nunca editar publicadas**.
9. WAL + foreign_keys ON desde el primer open.
10. Secrets **nunca** en SQLite / JSON / plano / logs.

### 6.3 Jobs

1. Persistir job **antes** de ejecutar.
2. No cola solo en memoria/React.
3. Generate job terminal = **`waiting_review`** (**no** `completed`) — **D-019**.
4. Approve/Reject son **posteriores** (Asset / Review).
5. Cancel cooperativa; cleanup solo tmp del job.
6. Idempotency key; recovery `interrupted`.
7. Progress en DB; no mentir “completed 0%” / “en Library”.

### 6.4 UX

1. Desktop first; sin overflow horizontal.
2. Preview principal ~70–75% cuando es la tarea; secundarios ≤30%.
3. Una acción primaria; siguiente acción evidente.
4. Empty / loading / error / success.
5. No menú primario de entidades.
6. Waiting Review no se presenta como Library.

### 6.5 Proceso IA (playbook)

1. Sin DoR completo → **no empezar**.
2. STOP rules → **no asumir**.
3. Orden de capas **fijo**; UI no primero en feature nueva.
4. Minimizar capas; si domain+persist+BE+FE → **dividir**.
5. Pruebas según **riesgo**.
6. Loop errores → loop simplificación.
7. Entrega con tests ejecutados/omitidos; **sin commit/push** salvo orden.
8. Revisión multi-rol antes de Done.

### 6.6 Testing por riesgo (resumen)

| Riesgo | Mínimo |
|--------|--------|
| LOW | Unit del cambio + smoke aplicable + `git diff --check` |
| MEDIUM | + Integration si hay I/O |
| HIGH | + E2E del flujo + regression si bugfix |
| ARCHITECTURE | Todas las aplicables + quality completa + docs/ADR |

### 6.7 Calidad Rust (desde ya)

- `cargo fmt`
- `cargo check`
- `cargo test`
- Clippy `-D warnings` cuando exista CI estable.

---

## 7. MVP — flujos y entidades (recordatorio)

### 7.1 Flujos

```text
Settings → Plans y/o Manual Factory → Factory → Review → Library
                              ↑                         ↓
                           Coverage ←───────────────────┘
```

| Flujo | Manual / Automatic notes |
|-------|---------------------------|
| Factory Manual | Lista de necesidades → FOUND/GENERATE → solo faltantes → Waiting Review |
| Factory Automatic | Theme → Plan approved → items → requests → Waiting Review |
| Review | Approve, Reject, Edit metadata, Regenerate, Mark duplicate |
| Library | Search/filter/export **approved only** |
| Coverage | Issues accionables (under/over, representations, themes, backlog) |
| Plans | Qué generar; approve habilita Automatic |
| Settings | Paths, providers, umbrales, jobs — sin producción |

### 7.2 Entidades de dominio (diseñadas, no codificadas)

Theme, Concept, Representation, Asset, GenerationRequest, CoveragePlan, CoveragePlanItem, AssetUsage, ExclusionRule, ConceptRelation (+ Job, Settings a nivel plataforma).

### 7.3 Cadena de valor

```text
Concepto → Representaciones → Assets → Uso
```

---

## 8. Plan de implementación (estado)

Fuente: [11-IMPLEMENTATION_PLAN.md](./11-IMPLEMENTATION_PLAN.md).

| Grupo | Fases (plan) | Estado |
|-------|--------------|--------|
| Fundación docs | 0 | Hecha (+ Foundation 0 playbook) |
| Scaffold monorepo | 1 | **Hecha** (shell + tests infra) |
| Plataforma SQLite/FS/jobs/settings | 2–5 | **No empezada** (negocio/pausas) |
| Dominio / read models / review / factory | 6–19 | No |
| UX cierre MVP / hardening | 20–24 | No |

**Post-MVP documentado:** provider real + OS secrets, import, FTS, thumbnails, **pHash**, VigilCut consumer, Tauri E2E, multi-provider (no OmniRoute completo), OmniRoute/cloud fuera.

---

## 9. Herramientas e infra detectadas

### 9.1 Presentes

| Área | Tool |
|------|------|
| Lenguajes | Rust 1.97, TypeScript ~5.7, Node 20+ |
| Desktop | Tauri 2, WebView2 (Windows) |
| UI | React 19, React Router 7, Vite 6 |
| Monorepo JS | pnpm 8 |
| Unit FE | Vitest 3 + Testing Library + jsdom |
| E2E | Playwright (config + tests shell) |
| Unit Rust | cargo test |
| Format/check Rust | rustfmt, cargo check (scripts) |
| IPC | `@tauri-apps/api` invoke |

### 9.2 Ausentes (huecos de tooling)

| Ausente | Nota |
|---------|------|
| ESLint | No instalado |
| Prettier | No instalado |
| GitHub Actions / CI | No hay `.github/workflows` |
| Clippy como gate CI | Decidido para cuando CI exista |
| rusqlite en Cargo | Solo diseñado |
| Job worker | Solo diseñado |
| OS keychain plugin | Para primer provider real (D-023) |
| E2E Tauri | Explicitamente posterior a Playwright+Vite |

---

## 10. Huecos detectados (completo)

### 10.1 Producto / proceso

| Hueco | Impacto |
|-------|---------|
| Convivencia **dos sets** de constituciones (F0 corto vs 00–09 largo) | Riesgo de contradicción si una IA lee solo el set viejo |
| Observability no tiene archivo corto F0 dedicado | Detalle solo en `08-OBSERVABILITY-CONSTITUTION.md` |
| P-001/P-003/P-004/P-005 abiertas | Pueden bloquear detalles de Factory/Review/Coverage |
| Negocio en pausa sin fecha de Foundation 1 | Correcto por decisión; requiere aprobación explícita |

### 10.2 Dominio / datos (diseño vs código)

| Hueco | Impacto |
|-------|---------|
| Cero tablas SQLite en runtime | No hay persistencia real aún |
| Cero MediaStore implementado | Paths/hash no enforzados en código |
| Cero jobs en DB | Durabilidad solo documentada |
| Esquema 05-DATABASE no migrado | Normal en pausa; al implementar respetar D-025 |

### 10.3 QA / E2E

| Hueco | Impacto |
|-------|---------|
| **Playwright E2E falló** en corrida previa: timeout 120s esperando Vite en `127.0.0.1:1420` | Infra E2E no está “verde de extremo a extremo” hasta arreglar `webServer`/host |
| `pnpm test:e2e:install` necesario por máquina | Documentado; no es automático en CI (no hay CI) |
| Sin regression suite de negocio | Esperable sin features |
| Sin tests de dominio de catálogo | Scaffold only (4 tests Rust smoke + 1 Vitest placeholder) |

### 10.4 Seguridad

| Hueco | Impacto |
|-------|---------|
| Sin integración keychain | OK hasta provider real; no improvisar JSON secrets |
| Capabilities Tauri mínimas actuales | Revisar al añadir FS real |

### 10.5 Documentación

| Hueco | Impacto |
|-------|---------|
| Algunos archivos 00–09 tuvieron riesgo de encoding en ediciones previas | Preferir F0 reescrito limpio |
| README vs START-HERE | Ambos existen; START-HERE ya prioriza F0 |
| No hay deprecación formal banner en cada 00–09 | Mitigado por START-HERE / este review |

### 10.6 Git / entrega

| Hueco | Impacto |
|-------|---------|
| Repo con mucho untracked / sin commits de fundación consolidados | Usuario controla commits (D-018) |
| Historial inicial solo README | Normal |

---

## 11. Riesgos globales de la fundación

| Riesgo | Severidad | Mitigación actual |
|--------|-----------|-------------------|
| IA ignora playbook y codea UI-first | Alta | AI_PLAYBOOK orden de capas + STOP rules |
| Docs viejos contradicen D-019 (generate completed) | Alta | 06-JOBS + 05-JOBS-CONSTITUTION + D-019 + F0 actualizados |
| Scope multi-capa en una PR | Alta | Playbook: dividir tareas |
| E2E rojo silencioso | Media | Hueco 10.3; arreglar en tarea infra |
| Doble constitución confunde | Media | START-HERE + este review: F0 gana |
| Empezar negocio sin SQLite/jobs | Alta | D-026 + orden de implementación |
| Secrets en settings table “temporal” | Alta | SECURITY + D-023 |

---

## 12. Criterios de “fundación lista”

| Criterio | Estado |
|----------|--------|
| Producto y non-goals documentados | Sí |
| Dominio diseñado (no codificado) | Sí |
| Arquitectura monorepo definida | Sí |
| Seis flujos y workflows | Sí |
| Decisiones ADR D-001…D-026 | Sí (D-008 superseded) |
| Metodología IA (playbook) | Sí |
| Constituciones F0 | Sí |
| Scaffold compilable | Sí |
| Tests infra (Vitest, cargo, Playwright config) | Sí (E2E runtime con gap de webServer) |
| SQLite/jobs en código | No (pausa) |
| Features de negocio | No (pausa) |
| Aprobación para Foundation 1 | **Pendiente** |

---

## 13. Recomendaciones inmediatas (sin implementar aquí)

1. **Aprobar Foundation 0** formalmente.
2. Opcional: banner “Deprecated — ver Foundation 0” en `constitution/00`–`09`.
3. **Tarea infra E2E** (riesgo LOW/MEDIUM): arreglar Playwright `webServer` (host `127.0.0.1`, timeout, o `vite --host`).
4. **Foundation 1** sugerida: SQLite open + WAL + FKs + migraciones + settings key-value + tests — **sin** Factory de negocio.
5. Resolver P-001 antes de Manual Factory.
6. No instalar dependencias “por si acaso”; solo por fase aprobada.
7. Cuando haya CI: `pnpm quality` + `test:e2e` + clippy `-D warnings`.

---

## 14. Inventario de archivos de docs (checklist)

### Foundation 0

- [x] `docs/AI_PLAYBOOK.md`
- [x] `docs/PRODUCT.md`
- [x] `docs/ARCHITECTURE.md`
- [x] `docs/constitution/ENGINEERING.md`
- [x] `docs/constitution/UX_UI.md`
- [x] `docs/constitution/TESTING.md`
- [x] `docs/constitution/SECURITY.md`
- [x] `docs/constitution/DONE.md`
- [x] `docs/00-START-HERE.md` (índice F0)
- [x] `docs/FOUNDATION_REVIEW.md` (este archivo)

### Diseño profundo 01–13

- [x] `01` … `13` presentes

### Constituciones 00–09

- [x] `constitution/00` … `09` presentes

---

## 15. Invariantes de una línea (cheat sheet)

```text
LOCAL · 6 FLUJOS · CONCEPT FIRST
PLANS=QUÉ · FACTORY=CÓMO
GENERATE JOB → waiting_review (NOT completed)
ASSET → Library ONLY via Approve
AUTOMATIC → plan approved only
SQLITE metadata + FS bytes
JOBS durable · no memory-only queue
SECRETS → OS store only
MVP HASH → SHA-256 only
NO VIGILCUT COUPLING · NO OMNROUTE IN CORE
UI NEVER FIRST LAYER OF A FEATURE
NO BUSINESS CODE UNTIL FOUNDATION 1 APPROVED
```

---

## 16. Conclusión

La fundación de Visual Library está **completa a nivel metodológico y de diseño**, con **scaffold técnico y tests base**, y **explícitamente sin lógica de negocio**.

| Capa de fundación | Completitud |
|-------------------|-------------|
| Producto / non-goals | Completa |
| Dominio / workflows (docs) | Completa |
| Arquitectura (docs) | Completa |
| Decisiones ADR | Completa hasta D-026 |
| Playbook + constituciones F0 | Completa |
| Scaffold app | Completa (placeholder) |
| Infra tests | Casi completa (E2E runtime gap) |
| Persistencia / jobs código | No iniciada (correcto bajo D-026) |
| Features MVP | No iniciadas (correcto bajo D-026) |

**Siguiente paso:** aprobación humana de esta fundación y, si procede, orden explícita de **Foundation 1** (plataforma de datos), no de pantallas de Factory.

---

*Fin de FOUNDATION_REVIEW.md*
