# ENGINEERING Constitution

**Autoridad de ingeniería (Foundation 0).**
Backend, frontend técnico, SQLite, FS, jobs, API.
UX visual → `UX_UI.md`. Seguridad de secrets/paths → `SECURITY.md` (no duplicar aquí reglas de privacidad).

---

## 1. Separación de responsabilidades

| Capa | Hace | No hace |
|------|------|---------|
| domain | Entidades, transiciones, políticas puras | I/O |
| application | Use cases, orquestación, ports | rusqlite/Tauri concretos |
| infrastructure | SQLite, FS, providers, worker | Nuevas reglas de negocio |
| api (commands) | Parse → use case → map error | SQL / invariantes largas |
| frontend | Presentación, IPC, estado efímero | Dominio, SQL, providers directos |

Dependencias: `api → application → domain`; `infrastructure → application → domain`.

---

## 2. Patrones

| Patrón | Uso |
|--------|-----|
| Ports & adapters | DB, FS, clock, ids, ImageProvider, JobQueue |
| Use case por acción de producto | Un command ≈ un caso de uso |
| Dual store | SQLite metadata; FS bytes |
| Jobs durables | Trabajo largo fuera del request UI síncrono |
| Idempotencia | `idempotency_key` / request id en reintentos |
| Stub first | Provider de imagen falso hasta fase real |

**No** abstracciones para necesidades no demostradas.

---

## 3. API / contratos

- Commands nombran **acciones de producto** (`approve_asset`, no `set_status`).
- Inputs validados en la frontera.
- Invariantes en dominio.
- Errores estructurados: `code`, `message`, `retryable`, `suggested_action`, `detail?`.
- Tipos estables FE ↔ BE; sin `any` permanente en TS.
- No exponer filas SQL crudas ni detalles de proveedor.

---

## 4. Dominio (implementación)

- Transiciones de Asset explícitas y testeadas.
- Generate **nunca** deja Asset `approved`.
- Job generate terminal = **`waiting_review`**.
- Automatic falla si plan ≠ `approved`.
- Plans no llaman ImageProvider.
- Sin `unwrap`/`expect`/`panic` en rutas productivas.
- Coste desconocido ≠ 0.

---

## 5. Persistencia SQLite

| Regla |
|-------|
| Migraciones **numeradas** (`0001_….sql`) |
| **Nunca** modificar migraciones **publicadas** |
| `PRAGMA foreign_keys=ON` desde el inicio |
| `PRAGMA journal_mode=WAL` desde el inicio |
| Transacciones cuando varios writes = una operación |
| Queries parametrizadas |
| Índices solo para consultas reales |
| No BLOB de imágenes en SQLite (salvo excepción aprobada) |
| No secretos en SQLite |

---

## 6. Filesystem

| Regla |
|-------|
| Media root + app data administrados |
| Asset: id, sha-256, path, size, mime, dimensions, status |
| No sobrescribir archivos finales |
| Temp → validar → promover |
| Anti path-traversal y symlink inseguro en cleanup |
| Cleanup limitado al workspace del job |
| Archivo faltante = estado controlado |

Duplicados MVP: **SHA-256** only.

---

## 7. Jobs

| Regla |
|-------|
| Persistir job **antes** de ejecutar |
| No cola solo en memoria / React |
| Campos mínimos: id, type, status, input, params, idempotency_key, attempts, progress, error, timestamps, outputs |
| Generate → **`waiting_review`** (no `completed`) |
| Approve/Reject = posteriores (Asset), no reescribir generate como Library done |
| Cancel: cooperativo + tmp propio + diagnóstico |
| Progress snapshots en DB (eventos no son verdad) |
| Recovery: `running` abandonado → `interrupted` → requeue/fail |
| Worker en proceso; concurrency default 1 en MVP |

---

## 8. Frontend técnico (no visual)

| Regla |
|-------|
| Código por `flows/<station>` |
| Estado: UI efímera vs snapshot persistido vs jobs (separados) |
| No store global monolítico como canonicidad |
| Mutaciones solo vía IPC |
| Anti doble envío |
| Refrescar snapshot tras eventos importantes |
| Reconstruir tras reload desde backend |
| Jobs solo por acción explícita (no side-effect de mount) |
| Extraer componentes genéricos solo con ≥2 usos reales |

---

## 9. Providers y adaptadores

- Port `ImageProvider`.
- Stub en tests y hasta fase real.
- Cambiar provider no cambia dominio.
- Secrets: ver `SECURITY.md` (OS store).
- Sin OmniRoute en el core.

---

## 10. Calidad Rust (herramientas reales)

Si la tarea toca Rust:

```text
pnpm fmt:rust      # cargo fmt --all
pnpm check:rust    # cargo check --workspace
pnpm test:rust     # cargo test --workspace
```

`cargo clippy -D warnings`: gate cuando exista CI estable.

---

## 11. Calidad TypeScript (herramientas reales)

```text
pnpm test:ui       # vitest
pnpm build:ui      # tsc --noEmit + vite build
pnpm test:e2e      # playwright + vite (si aplica)
```

No ESLint/Prettier en el repo hoy.

---

## 12. Anti-patrones

- SQL en domain o frontend
- Generate → job `completed`
- Feature en 4 capas sin dividir
- Editar migración publicada
- Secrets en settings JSON
- `unwrap` en approve/generate
- Placeholder vendido como Done

---

## 13. Referencias

- `docs/ARCHITECTURE.md` · `docs/AI_PLAYBOOK.md`
- `docs/12-DECISIONS.md` (D-019…D-026)
