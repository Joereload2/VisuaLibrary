# 03 — ARCHITECTURE

> **Referencia ampliada — no normativa.** Autoridad de arquitectura: [`ARCHITECTURE.md`](./ARCHITECTURE.md).

## 1. Propósito

Definir la arquitectura del sistema **local** Visual Library: límites, capas, estructura de monorepo, dependencias y principios de evolución.

**Sin código de negocio. Sin mover archivos todavía.**

---

## 2. Resumen ejecutivo

| Decisión | Elección |
|----------|----------|
| Tipo de app | Desktop local (no SaaS) |
| Shell | **Tauri 2** |
| Core | **Rust** (dominio + application + infrastructure) |
| UI | **TypeScript** frontend embebido (por flujos) |
| Datos estructurados | **SQLite** |
| Binarios | **Filesystem administrado** |
| Jobs | Durables en SQLite + worker en proceso |
| Multiplayer / cloud DB | No |
| Acoplamiento VigilCut | No (solo futuro consumer vía export/usage) |

---

## 3. Principios arquitectónicos

1. **Local-first:** el núcleo funciona offline.
2. **DDD pragmático:** dominio puro sin I/O; infraestructura al borde.
3. **Flujos sobre tablas:** módulos de aplicación alineados a Factory, Review, Library, Coverage, Plans, Settings.
4. **Hexagonal / ports & adapters:** UI y Tauri commands son adapters.
5. **Jobs durables:** ningún trabajo largo “solo en RAM”.
6. **Separación Plan / Factory:** enforced en application services.
7. **Library gate:** enforced en dominio + queries.
8. **Independencia de producto:** crates y paquetes sin imports de VigilCut.
9. **Incrementalidad:** fases pequeñas (ver plan de implementación).
10. **Testabilidad:** dominio y application testables sin Tauri UI.

---

## 4. Diagrama de contexto (C4-L1)

```
┌─────────────────────────────────────────────────────────┐
│                     Usuario local                        │
└───────────────────────────┬─────────────────────────────┘
                            │ UI
┌───────────────────────────▼─────────────────────────────┐
│                   Visual Library (Tauri)                 │
│  Frontend (flujos)  │  Rust core  │  SQLite  │  FS media │
└───────────┬─────────────────┬───────────────────────────┘
            │                 │
            │ providers       │ (futuro) export / usage refs
            ▼                 ▼
   [Image providers]    [Consumers: VigilCut, …]
   (APIs opcionales)     (fuera del proceso VL)
```

- Los proveedores de imagen son **adapters opcionales** configurados en Settings.
- En fases tempranas pueden existir **stubs/fakes** para no bloquear el dominio.
- **No** hay Supabase/PostgreSQL/cloud como source of truth.

---

## 5. Contenedores (C4-L2)

| Contenedor | Tecnología | Rol |
|------------|------------|-----|
| **UI WebView** | TS + framework UI | Estaciones de trabajo, estado de presentación |
| **Tauri Host** | Rust | Windowing, IPC, permisos FS, ciclo de vida app |
| **Application Core** | Rust crates | Casos de uso por flujo |
| **Domain** | Rust crate | Entidades, invariantes, políticas |
| **SQLite** | archivo local | Metadata, jobs, planes, estados |
| **Media Store** | directorios locales | Binarios de assets |
| **Job Worker** | thread/task en proceso | Ejecuta jobs durables |

Todo vive en **un solo producto instalable**.

---

## 6. Capas y dependencias

```
ui (TypeScript)
    │  IPC / Tauri commands
    ▼
apps/desktop (Tauri commands = adapters)
    │
    ▼
application  ──►  domain   (sin I/O)
    │
    ▼
infrastructure (sqlite, fs, providers, clock, id gen)
```

**Reglas de dependencia:**

- `domain` no depende de nadie del proyecto.
- `application` depende de `domain` + traits (ports).
- `infrastructure` implementa ports.
- `apps/desktop` compone el grafo y expone commands.
- `ui` solo habla por contracts IPC versionados (tipos generados o schema compartido).

---

## 7. Módulos de aplicación (por flujo)

| Módulo application | Casos de uso principales (diseño) |
|--------------------|-----------------------------------|
| `factory_manual` | Import needs list, resolve FOUND/GENERATE, enqueue generate |
| `factory_automatic` | Load approved plan, materialize requests, enqueue generate |
| `review` | List waiting, approve, reject, edit metadata, regenerate, mark duplicate |
| `library` | Search/filter approved, get detail, export info, record usage |
| `coverage` | Compute issues, rankings, under/over coverage |
| `plans` | CRUD draft plans/items, approve, archive |
| `settings` | Read/write config, validate paths, providers |
| `jobs` | Enqueue, run, recover interrupted, cancel |

Los nombres de carpetas UI y commands deben reflejar **estos flujos**, no `concepts_page`.

---

## 8. Estructura definitiva propuesta del monorepo

Estado actual:

```
VisuaLibrary/
  README.md
  docs/
  .git/
```

**Propuesta (no aplicar aún):**

```
VisuaLibrary/
  README.md
  docs/                          # fundación (ya)
  AGENTS.md                      # opcional: reglas para agentes
  Cargo.toml                     # workspace Rust
  package.json                   # workspace JS (si monorepo npm/pnpm)
  pnpm-workspace.yaml            # o equivalente

  apps/
    desktop/                     # crate Tauri + shell
      src-tauri/
        Cargo.toml
        src/
          main.rs
          lib.rs
          commands/              # adapters IPC por flujo
          bootstrap.rs           # composition root
      # frontend embebido o apunta a packages/ui

  crates/
    domain/                      # puro
      src/
        concept/
        representation/
        asset/
        generation/
        coverage/
        plan/
        usage/
        exclusion/
        relation/
        common/                  # ids, errors, value objects
    application/                 # casos de uso + ports
      src/
        factory/
        review/
        library/
        coverage/
        plans/
        settings/
        jobs/
        ports/                   # traits: repos, fs, clock, providers
    infrastructure/
      src/
        sqlite/                  # migrations, repos
        fs_media/                # media store
        jobs_worker/
        providers/               # stubs + real adapters later
        config/
    ipc_contract/                # opcional: tipos DTO compartidos / schema
      ...

  packages/
    ui/                          # frontend TypeScript
      src/
        app/                     # shell, router por flujos
        flows/
          factory/
          review/
          library/
          coverage/
          plans/
          settings/
        shared/                  # design system mínimo, no pantallas inventadas
    ui-test/                     # e2e helpers

  data/                          # NO commitear datos de usuario
    .gitkeep

  fixtures/                      # datasets de prueba (sin binarios pesados)
  scripts/                       # dev tooling
  .gitignore
```

### 8.1 Qué NO va en la estructura

- Carpetas de navegación `pages/concepts`, `pages/assets` como IA de producto.
- `services/vigilcut` en el core.
- `supabase/`, `docker-compose` de Postgres para el producto.
- Lógica de IA / OmniRoute en crates del MVP.

### 8.2 Ubicación de datos en runtime (diseño)

Usar directorio de datos de la app (Tauri `app_data_dir` o path configurado en Settings):

```
{app_data}/visual-library/
  db/
    visual_library.sqlite
  media/
    assets/
      {yyyy}/
        {mm}/
          {asset_id}.{ext}
  exports/
  logs/
  tmp/
    jobs/
```

SQLite guarda paths **relativos** a `media/` cuando sea posible.

---

## 9. IPC (Tauri commands) — contrato por flujo

Principios:

- Commands **finos**, alineados a casos de uso.
- Sin SQL desde el frontend.
- Errores tipados y estables.
- Paginación en listados.
- Idempotencia donde aplique (approve dos veces = misma verdad).

**Grupos de commands (nombres ilustrativos, no implementación):**

| Grupo | Ejemplos de operaciones |
|-------|-------------------------|
| `factory` | `preview_manual_batch`, `submit_manual_batch`, `run_automatic_from_plan`, `list_batches` |
| `review` | `list_waiting_review`, `approve_asset`, `reject_asset`, `edit_asset_metadata`, `regenerate_asset`, `mark_duplicate` |
| `library` | `search_assets`, `get_asset`, `export_asset_info`, `record_usage` |
| `coverage` | `get_coverage_summary`, `list_coverage_issues` |
| `plans` | `list_plans`, `create_plan`, `add_plan_item`, `approve_plan`, `archive_plan` |
| `settings` | `get_settings`, `update_settings`, `validate_media_root` |
| `jobs` | `list_jobs`, `get_job`, `cancel_job`, `retry_job` |

---

## 10. Persistencia — rol de cada store

| Store | Guarda | No guarda |
|-------|--------|-----------|
| SQLite | entidades, estados, jobs, índices de búsqueda básicos | blobs pesados |
| Filesystem | binarios de media, exports, logs | verdad de estado de review |
| Memoria | caches UI, handles de worker | única copia de job state |

Detalle de esquema: [05-DATABASE.md](./05-DATABASE.md).

---

## 11. Concurrencia y consistencia

- **Un writer SQLite** principal (connection pool con cuidado; preferir serialización de escrituras de dominio).
- Transacciones por caso de uso.
- Job worker: reclama jobs con `UPDATE … WHERE status='queued'` y heartbeat; al arrancar marca `running` antiguos como `interrupted` y política de recovery.
- FS: escritura a `tmp/` + rename atómico al path final cuando sea posible.
- Hash de contenido tras escribir binario.

---

## 12. Seguridad local (MVP)

- Sin auth cloud.
- Secrets de proveedores en almacenamiento local de la app (OS keychain si Tauri lo permite; si no, archivo restringido — decisión en 12).
- Path traversal: validar que `storage_path` no salga del media root.
- Frontend no recibe paths absolutos arbitrarios sin validación backend.

---

## 13. Observabilidad

- Logs locales estructurados (tracing en Rust).
- Tabla o archivo de job attempts.
- Sin telemetría cloud en MVP (non-goal salvo opt-in futuro).

---

## 14. Evolución y versionado

- Migraciones SQLite versionadas (`schema_migrations`).
- Contratos IPC versionados (campos nuevos opcionales; no romper commands sin migración UI).
- App semver de producto independiente de VigilCut.

---

## 15. Alternativas consideradas

| Alternativa | Por qué se descarta (MVP) |
|-------------|---------------------------|
| Electron | Mayor peso; Tauri alinea con Rust core |
| Solo web local (browser) | Peor integración FS y empaquetado desktop |
| PostgreSQL local | Overkill; SQLite suficiente y portable |
| Supabase/cloud | Viola local-first y no-goals |
| Monolito UI-driven sin domain crate | Pierde invariantes y testabilidad |
| Microservicios | Complejidad injustificada en desktop local |

---

## 16. Riesgos arquitectónicos

| Riesgo | Mitigación |
|--------|------------|
| UI se convierte en CRUD de tablas | Navegación solo por 6 flujos; code review de rutas |
| Jobs solo en memoria | Tabla jobs + recovery al boot (Fases tempranas) |
| Acoplamiento a un provider de IA | Port `ImageProvider`; stubs primero |
| Paths rotos al mover media root | Paths relativos + herramienta de rebind en Settings (post) |
| Scope creep OmniRoute/VigilCut | 13-NON_GOALS + gates de fase |

---

## 17. Criterios de arquitectura “sólida” antes de features

1. Workspace Rust + app Tauri arrancable (hello).
2. SQLite + migraciones.
3. Media FS root configurable.
4. Job pipeline mínimo (echo job).
5. Un command por flujo “health/list empty”.
6. Tests de dominio de invariantes clave.
7. Docs alineadas (este paquete).

Eso corresponde a **Fases 0–2** del plan; **no** a Factory real con IA.

---

## 18. Referencias internas

- Dominio: [02-DOMAIN.md](./02-DOMAIN.md)
- Workflows: [04-WORKFLOWS.md](./04-WORKFLOWS.md)
- Backend: [08-BACKEND.md](./08-BACKEND.md)
- Frontend: [07-FRONTEND.md](./07-FRONTEND.md)
- Plan: [11-IMPLEMENTATION_PLAN.md](./11-IMPLEMENTATION_PLAN.md)
