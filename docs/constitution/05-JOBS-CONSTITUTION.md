> **LEGACY — no normativo.** Fuente de verdad: [README.md](./README.md) y Foundation 0 (`docs/AI_PLAYBOOK.md`, `PRODUCT.md`, `ARCHITECTURE.md`, `ENGINEERING.md`, `UX_UI.md`, `TESTING.md`, `SECURITY.md`, `DONE.md`). Si hay conflicto, **ignorar este archivo**.

# 05 â€” Jobs Constitution

**Estado:** Normativo
**Ãmbito:** Cola durable, worker en proceso, progreso, cancelaciÃ³n, recovery
**Realidad del repo:** especificado en `docs/06-JOBS.md`; worker y tabla jobs aÃºn no implementados (implementaciÃ³n de negocio en pausa).

---

## 1. Principio

Todo trabajo largo es **durable** en SQLite.
El worker Tauri/Rust en proceso es ejecutor, no fuente de verdad.
La UI muestra progreso y gobierna retry/cancel; **no** posee la cola.

---

## 2. Estados mÃ­nimos

```text
queued
running
waiting_review
completed
failed
interrupted
cancelling
cancelled
```

### Mapeo con el dominio Visual Library

| SituaciÃ³n | Job status | Entidad de negocio |
|-----------|------------|---------------------|
| Generate en cola / ejecutando | `queued` / `running` | GenerationRequest activo |
| Generate terminÃ³ y creÃ³ asset para humanos | **`waiting_review`** | Asset = `waiting_review` |
| Ã‰xito sin espera humana (p.ej. export) | `completed` | outputs vÃ¡lidos |
| Fallo | `failed` | error estructurado |
| Crash / kill | `interrupted` â†’ requeue o fail | recovery al boot |
| Usuario cancela | `cancelling` â†’ `cancelled` | cleanup tmp |

### Jobs de generaciÃ³n (APROBADO â€” D-019)

| # | Ley |
|---|-----|
| J-G1 | Los jobs de **generaciÃ³n** terminan en **`waiting_review`**. **No** en `completed`. |
| J-G2 | La **aprobaciÃ³n** o **rechazo** son transiciones **posteriores** (use case Review sobre el Asset). |
| J-G3 | Approve/Reject **no** reescriben en silencio el historial del job de generate como si hubiera sido `completed` de Library. |
| J-G4 | La UI **nunca** presenta un generate en `waiting_review` como â€œcompletado / en Libraryâ€. |
| J-G5 | Jobs que **no** requieren revisiÃ³n humana (export, echo, recompute) pueden usar `completed` con outputs vÃ¡lidos. |

---

## 3. Reglas generales

| # | Regla |
|---|--------|
| J-1 | **Persistir el job antes** de ejecutarlo. |
| J-2 | No mantener jobs Ãºnicamente en memoria. |
| J-3 | Todo job debe tener al menos: |

```text
id
type
status
input
parameters (o equivalente en payload)
idempotency_key
attempts
progress
error (estructurado, nullable)
timestamps (created/started/heartbeat/finishedâ€¦)
outputs (nullable hasta Ã©xito terminal del tipo)
```

| J-4 | Un job de generate en `waiting_review` debe tener outputs vÃ¡lidos (p.ej. `asset_id`, path) que permitan Review. |
| J-5 | Un reinicio convierte trabajos abandonados en `interrupted` o los devuelve de forma segura a `queued`. |
| J-6 | El **retry** es explÃ­cito y seguro (`idempotency_key`). |
| J-7 | Una **idempotency key** impide trabajo duplicado accidental. |
| J-8 | La **cancelaciÃ³n**: detiene cooperativamente Â· limpia solo temporales propios Â· conserva diagnÃ³stico. |
| J-9 | Los **eventos de progreso no son** la fuente de verdad. |
| J-10 | El progreso debe persistir **snapshots** suficientes para reconstruir la UI. |
| J-11 | No mostrar Ã©xito terminal con 0% de progreso. |
| J-12 | Un `queued` prolongado debe ofrecer **explicaciÃ³n diagnÃ³stica**. |
| J-13 | MVP: worker **en proceso**, concurrency default 1 salvo Settings + tests. |
| J-14 | Automatic materialize revalida plan `approved` al ejecutar. |
| J-15 | `generate` **nunca** deja Asset `approved`. |

---

## 4. Tipos de job (diseÃ±o)

| type | Status terminal tÃ­pico |
|------|------------------------|
| `generate_asset` | **`waiting_review`** |
| `manual_batch_resolve` | `completed` (decisiones FOUND/GENERATE) o falla |
| `automatic_plan_materialize` | `completed` (requests creados) o falla |
| `export_manifest` | `completed` |
| `coverage_recompute` | `completed` |
| `echo` | `completed` (scaffold/tests) |

Prohibido: OmniRoute DAG, workers multi-mÃ¡quina, cola Redis en MVP.

---

## 5. RelaciÃ³n con flujos

```
Plans.approve     â†’ no encola generate
Factory           â†’ persiste job generate â†’ running â†’ waiting_review + Asset waiting_review
Review humano     â†’ approve | reject | â€¦ (transiciones de Asset posteriores)
UI                â†’ list / cancel / retry via commands
```

---

## 6. Testing jobs (cuando exista implementaciÃ³n)

Obligatorio:

- persist before run
- generate termina en `waiting_review` (no `completed`)
- crash â†’ interrupted/recovery
- cancel queued sin side effects
- idempotency key
- generate no aprueba Asset
- progress snapshot reconstruible

Herramienta: `cargo test` en infrastructure/application.

---

## 7. Anti-patrones

- Cola en `useState`
- Generate job â†’ `completed`
- Asset `approved` dentro del handler generate
- Retry que duplica assets
- Cancel que borra fuera de tmp del job
- Progress solo por eventos sin DB

---

## 8. Checklist PR jobs

- [ ] Persistido antes de run
- [ ] Generate â†’ `waiting_review`
- [ ] Campos mÃ­nimos
- [ ] Idempotency
- [ ] Cancel/cleanup
- [ ] Recovery
- [ ] Tests del riesgo

---

## 9. Referencias

- `docs/06-JOBS.md` Â· `docs/04-WORKFLOWS.md` Â· `docs/12-DECISIONS.md` (D-019)
- [03-BACKEND-CONSTITUTION.md](./03-BACKEND-CONSTITUTION.md) Â· [04-DATA-CONSTITUTION.md](./04-DATA-CONSTITUTION.md)
