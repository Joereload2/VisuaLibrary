> **LEGACY — no normativo.** Fuente de verdad: [README.md](./README.md) y Foundation 0 (`docs/AI_PLAYBOOK.md`, `PRODUCT.md`, `ARCHITECTURE.md`, `ENGINEERING.md`, `UX_UI.md`, `TESTING.md`, `SECURITY.md`, `DONE.md`). Si hay conflicto, **ignorar este archivo**.

# 03 Ã¢â‚¬â€ Backend Constitution

**Estado:** Normativo
**ÃƒÂmbito:** `crates/*`, `apps/desktop/src-tauri`
**Stack real:**

| Crate / app | Nombre Cargo | Rol hoy |
|-------------|--------------|---------|
| domain | `visual_library_domain` | Scaffold + tests mÃƒÂ­nimos |
| application | `visual_library_application` | Scaffold |
| infrastructure | `visual_library_infrastructure` | Scaffold |
| desktop | `visual_library_desktop` | Tauri 2 shell + command `health` |
| Errors | `thiserror` (workspace) | Presente |
| Serde | `serde` / `serde_json` | Presente |
| SQLite | Ã¢â‚¬â€ | **No** en Cargo aÃƒÂºn (fase posterior) |

---

## 1. Capas y direcciÃƒÂ³n de dependencias

```
commands / Tauri IPC  (apps/desktop/src-tauri)
        Ã¢â€ â€œ
   application        (casos de uso + ports)
        Ã¢â€ â€œ
     domain           (invariantes puras)

application
    Ã¢â€ â€œ
infrastructure:
    persistence (SQLite) Ã‚Â· providers Ã‚Â· filesystem Ã‚Â· jobs worker
```

| # | Ley |
|---|-----|
| B-1 | El dominio **no** depende de: UI Ã‚Â· Tauri Ã‚Â· SQLite Ã‚Â· filesystem Ã‚Â· proveedor externo Ã‚Â· variables de entorno. |
| B-2 | `application` no importa rusqlite/Tauri concretos; solo ports. |
| B-3 | `infrastructure` implementa ports; no redefine reglas de negocio. |
| B-4 | Commands son adapters delgados: parse Ã¢â€ â€™ use case Ã¢â€ â€™ map error. |
| B-5 | Sin crates ni tipos de VigilCut en el core. |

---

## 2. Casos de uso (API de producto)

La API (commands Tauri) expresa **acciones del producto**.

**Preferir:**

```text
create_generation_request
approve_asset
reject_asset
search_library
register_asset_usage
preview_manual_batch
submit_manual_batch
run_automatic_from_plan
approve_coverage_plan
list_waiting_review
```

**Evitar APIs fragmentadas:**

```text
set_status
update_row
run_sql
set_path
increment_counter
```

| # | Ley |
|---|-----|
| B-10 | Un command = un caso de uso (o query) con contrato estable. |
| B-11 | No exponer SQL ni filas crudas como API pÃƒÂºblica. |

---

## 3. Reglas de implementaciÃƒÂ³n

| # | Regla |
|---|--------|
| B-20 | Validar **inputs** en la frontera (command / DTO). |
| B-21 | Validar **invariantes** en el dominio. |
| B-22 | Usar **transacciones** cuando varios cambios formen una sola operaciÃƒÂ³n. |
| B-23 | No silenciar errores (`let _ = Ã¢â‚¬Â¦` en fallos de negocio). |
| B-24 | No usar `unwrap`, `expect` o `panic` en rutas productivas. (Tests y build scripts pueden ser mÃƒÂ¡s estrictos; production paths no.) |
| B-25 | Errores **estructurados** (cÃƒÂ³digo + mensaje + retryable + detalle). |
| B-26 | Proveedores externos detrÃƒÂ¡s de **adaptadores** (`ImageProvider` port). |
| B-27 | La lÃƒÂ³gica de negocio **no** depende de OmniRoute. |
| B-28 | Cambiar de proveedor **no** cambia el dominio. |
| B-29 | Coste desconocido **no** equivale a cero (modelar unknown vs 0). |
| B-30 | No registrar secretos. |
| B-31 | Operaciones repetibles **idempotentes** cuando aplique (keys, request ids). |

---

## 4. Invariantes de producto (backend enforce)

| # | Invariante |
|---|------------|
| B-40 | Generate crea asset en revisiÃƒÂ³n humana; **nunca** `approved` automÃƒÂ¡tico. |
| B-41 | Automatic solo con CoveragePlan `approved`. |
| B-42 | Plans no escriben binarios ni llaman providers. |
| B-43 | Library search/export solo `approved`. |
| B-44 | FOUND solo sobre approved + polÃƒÂ­tica de matching. |
| B-45 | Regenerate: no mutar binario en silencio (supersede + nuevo asset). |

---

## 5. Bootstrap (cuando exista persistencia)

Orden de arranque del host Tauri:

1. Cargar settings / paths
2. Abrir SQLite + migraciones
3. Asegurar directorios media/tmp
4. Recovery de jobs
5. Iniciar worker en proceso
6. Registrar commands

Hoy (Fase 1): solo `health` + ventana Ã¢â‚¬â€ el orden completo es norma para fases Ã¢â€°Â¥ 2Ã¢â‚¬â€œ6.

---

## 6. Testing backend (herramienta real)

```text
cargo test --workspace
cargo test -p visual_library_domain
cargo test -p visual_library_application
cargo test -p visual_library_infrastructure
cargo test -p visual_library_desktop
```

| # | Ley |
|---|-----|
| B-50 | Invariantes nuevas Ã¢â€ â€™ test de dominio o application que falle si se revierte. |
| B-51 | Sin SQLite aÃƒÂºn, los tests de persistencia se aÃƒÂ±aden **con** la fase de SQLite, no se fingen. |

---

## 7. Anti-patrones

- SQL en `domain`
- Command de 200 lÃƒÂ­neas con reglas de coverage
- `unwrap()` en handler de approve
- Feature flag que salta Review en release
- Dependencia de OmniRoute en application

---

## 8. Checklist PR backend

- [ ] Dependencias de capas OK
- [ ] Use case de producto (no set_status)
- [ ] Errores estructurados
- [ ] Sin unwrap/expect productivos nuevos
- [ ] TransacciÃƒÂ³n si multi-write
- [ ] Idempotencia si reintento
- [ ] `cargo test` del crate tocado

---

## 9. Referencias

- `docs/08-BACKEND.md` Ã‚Â· `docs/02-DOMAIN.md` Ã‚Â· `docs/03-ARCHITECTURE.md`
- [04-DATA-CONSTITUTION.md](./04-DATA-CONSTITUTION.md) Ã‚Â· [05-JOBS-CONSTITUTION.md](./05-JOBS-CONSTITUTION.md)
