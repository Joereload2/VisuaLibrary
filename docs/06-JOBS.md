# 06 — JOBS

## 1. Propósito

Definir el sistema de **trabajos durables** de Visual Library: estados, tipos, recovery, relación con dominio y criterios de aceptación.

Todo trabajo no trivial debe sobrevivir reinicios de la aplicación.

---

## 2. Principio rector

> **No usar estados solamente en memoria.**

La cola de trabajo, el estado actual y el resultado (o error) viven en **SQLite**.
El worker en proceso es un ejecutor, no la source of truth.

---

## 3. Estados del Job (infraestructura)

Estados requeridos por producto:

| Estado | Significado |
|--------|-------------|
| `queued` | Listo para ejecutarse (o programado). |
| `running` | Worker lo reclamó y está en progreso. |
| `waiting_review` | *(ver §3.1)* |
| `completed` | Terminó con éxito. |
| `failed` | Falló de forma terminal o agotó reintentos. |
| `cancelled` | Cancelado por usuario o sistema. |
| `interrupted` | App cerró o worker murió mientras `running`. |

### 3.1 Sobre `waiting_review` en Jobs (D-019 — aprobado)

**Decisión vigente:** los jobs de **generación** terminan en **`waiting_review`**, **no** en `completed`.

| Hecho | Status |
|-------|--------|
| Job `generate_asset` final exitoso | **`waiting_review`** |
| Asset creado | **`waiting_review`** |
| Approve / Reject | Transiciones **posteriores** del Asset (Review), no “completar” el generate como Library |

Razones:

- El usuario y la UI no deben ver generate como “completed / en Library”.
- Approve/Reject son acciones humanas posteriores.
- El worker no se bloquea esperando al humano: el job ya está en estado terminal de generación (`waiting_review`); el humano actúa sobre el Asset.

Jobs **sin** revisión humana (export, echo, materialize de plan, etc.) pueden usar `completed` con outputs válidos.

Ver también: `docs/constitution/05-JOBS-CONSTITUTION.md`, `docs/12-DECISIONS.md` D-019.

---

## 4. Ciclo de vida

```
        enqueue
           │
           ▼
        queued ──────────────► cancelled
           │
           │ claim
           ▼
        running ──── heartbeat ──┐
           │                     │
           ├─ generate OK ──► waiting_review   (D-019)
           ├─ other OK ──► completed
           ├─ error retryable ──► queued (attempts++)
           ├─ error terminal ──► failed
           ├─ cancel ──► cancelled
           └─ process death ──► interrupted ──► queued | failed
```

### 4.1 Recovery al arranque

1. `UPDATE jobs SET status='interrupted' WHERE status='running'`.
2. Para cada interrupted:
   - si `attempts < max_attempts` y job es idempotente/reintentable → `queued`
   - si no → `failed` con mensaje de interrupted
3. Reanudar worker.

### 4.2 Heartbeat

- Mientras `running`, actualizar `heartbeat_at` cada N segundos.
- Watchdog opcional: si heartbeat viejo, marcar `interrupted` (MVP: al menos al boot).

---

## 5. Modelo de datos (resumen)

Tabla `jobs` (detalle columnas en [05-DATABASE.md](./05-DATABASE.md)):

- Identidad, tipo, payload JSON
- status, priority, attempts, max_attempts
- scheduled_at, started_at, finished_at, heartbeat_at
- last_error
- related_entity_type / related_entity_id
- timestamps

Tabla `job_events` para traza de transiciones (recomendado desde fase de jobs).

---

## 6. Tipos de Job del MVP (diseño)

| job_type | Payload esencial | Resultado de éxito | Reintentable |
|----------|------------------|--------------------|--------------|
| `manual_batch_resolve` | batch_id | requests con decision FOUND/GENERATE | parcial |
| `automatic_plan_materialize` | plan_id, batch_id | requests creados | sí (idempotente por item) |
| `generate_asset` | generation_request_id | asset_id waiting_review + storage_path | sí con cuidado |
| `export_manifest` | export_id, asset_ids | path export | sí |
| `coverage_recompute` | scope opcional | cache/issues materializados (si se cachean) | sí |
| `echo` / `health_probe` | message | same | sí (solo scaffold) |

**No incluir en MVP de jobs:** entrenamiento de modelos, sync cloud, pipelines VigilCut.

---

## 7. Relación Job ↔ Dominio

```
CoveragePlan (approved)
    └── automatic_plan_materialize (job)
            └── GenerationRequest(s)
                    └── generate_asset (job)
                            └── Asset(status=waiting_review)
                                    └── Review (humano, no job runner)
                                            └── Asset(approved) ∈ Library
```

Manual:

```
Needs list
    └── manual_batch_resolve (job o use-case sync + jobs generate)
            └── GenerationRequest decision
                    └── generate_asset (job) …
```

**Reglas:**

1. Automatic materialize exige plan `approved`.
2. `generate_asset` no marca asset `approved`.
3. Cancel de job `generate_asset` en `queued` no crea asset.
4. Si falla a mitad de escritura FS: no dejar Asset `approved`; limpiar tmp; request `failed`.

---

## 8. Idempotencia y side effects

| Job | Clave de idempotencia | Comportamiento |
|-----|----------------------|----------------|
| `generate_asset` | `generation_request_id` | Si ya hay `result_asset_id`, no regenerar |
| `automatic_plan_materialize` | `(plan_id, run_token)` o item markers | No duplicar requests abiertos para mismo item |
| `export_manifest` | `export_id` | Reescribir export folder de forma segura |

Los providers de imagen pueden no ser idempotentes: por eso la idempotencia se ancla en **nuestro** `generation_request_id`.

---

## 9. Concurrencia

| Parámetro | Propuesta MVP |
|-----------|----------------|
| Workers | 1 (simplifica SQLite + providers) |
| Concurrency de generate | configurable 1–N en Settings (default 1) |
| Claim | transacción `BEGIN IMMEDIATE` |
| Prioridad | menor número = más prioritario; generate default 100; user cancel high |

---

## 10. API de aplicación (ports)

```
trait JobRepository {
  enqueue(...)
  claim_next(...)
  heartbeat(id)
  complete(id, result)
  fail(id, error, retryable)
  cancel(id)
  mark_interrupted_running()
  list(filter)
  get(id)
}

trait JobWorker {
  start()
  stop()
}

trait JobHandler {
  job_type() -> &str
  handle(payload, ctx) -> Result<JobResult>
}
```

Handlers viven en `application` o `infrastructure` según dependencias; el registry en composition root.

---

## 11. Errores y reintentos

| Clase de error | Acción |
|----------------|--------|
| Network/provider transient | retry con backoff; requeue |
| Validation / domain reject | failed sin retry |
| FS permission | failed; mensaje accionable en Settings |
| Cancelación | cancelled |
| Panic en handler | interrupted/failed; log |

`max_attempts` default: 3 (configurable).

---

## 12. Observabilidad

- `job_events` por transición
- logs tracing con `job_id`, `job_type`, `related_entity_id`
- UI Settings o panel mínimo: listado de jobs recientes (puede vivir bajo Settings o Factory summary — **no** es un 7º flujo; es widget de soporte dentro de Factory/Settings)

---

## 13. Criterios de aceptación (Jobs)

1. Encolar un job, matar el proceso en `running`, reiniciar → job no se pierde (`interrupted` → requeue o failed explícito).
2. `generate_asset` exitoso deja Asset y Job en `waiting_review` (no `completed`).
3. Cancel de `queued` impide side effects.
4. No existen colas **solo** en `HashMap`/memoria como única verdad.
5. Reintento de `generate_asset` con request ya completado no crea segundo asset.
6. Worker no ejecuta `automatic_plan_materialize` si el plan dejó de estar approved (revalidar al correr).

---

## 14. Fases de implementación relacionadas

Ver [11-IMPLEMENTATION_PLAN.md](./11-IMPLEMENTATION_PLAN.md):

- Fase scaffold: tabla jobs + echo job
- Fase factory: generate_asset con provider stub
- Fase recovery: interrupted tests

---

## 15. No-goals de jobs

- Cluster de workers multi-máquina
- Cola Redis/SQS
- Cron cloud
- Orchestration tipo OmniRoute
- DAGs complejos con UI de pipeline

---

## 16. Referencias

- Workflows: [04-WORKFLOWS.md](./04-WORKFLOWS.md)
- Database: [05-DATABASE.md](./05-DATABASE.md)
- Backend: [08-BACKEND.md](./08-BACKEND.md)
