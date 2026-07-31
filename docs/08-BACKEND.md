# 08 — BACKEND

## 1. Propósito

Definir la arquitectura del backend **Rust** de Visual Library: crates, puertos, casos de uso por flujo, Tauri adapters y reglas de composición.

**Sin implementación de lógica de negocio en esta entrega.**

---

## 2. Rol del backend

El backend es el **dueño de las invariantes**:

- Library gate (solo approved)  
- Generate → waiting_review  
- Automatic solo con plan approved  
- Plans no generan binarios  
- Jobs durables  
- Paths seguros bajo media root  
- Matching FOUND/GENERATE  

---

## 3. Crates del workspace

| Crate | Responsabilidad | Dependencias permitidas |
|-------|-----------------|-------------------------|
| `domain` | Entidades, VOs, políticas, errores de dominio | std + libs puras (uuid/ulid, thiserror, time) |
| `application` | Use cases, DTOs de app, ports (traits) | domain |
| `infrastructure` | SQLite, FS, providers, job worker, config | application, domain, crates I/O |
| `apps/desktop` (src-tauri) | Tauri commands, bootstrap, wiring | application, infrastructure, domain |

Opcional: `ipc_contract` para tipos compartidos generados.

### 3.1 Regla de dependencia

```
desktop → application → domain
desktop → infrastructure → application → domain
infrastructure ↛ desktop
domain ↛ application/infrastructure
```

---

## 4. Domain crate — diseño de módulos

```
domain/src/
  lib.rs
  ids.rs
  error.rs
  clock.rs                 # trait o types de tiempo puros
  concept/
    mod.rs
    entity.rs
    policy.rs
  representation/
  asset/
    mod.rs
    entity.rs
    status.rs              # transitions
    review.rs
  generation/
    request.rs
    found_policy.rs
  plan/
    plan.rs
    item.rs
  coverage/
    issue.rs
    targets.rs
  usage/
  exclusion/
  relation/
  theme/
```

**Contenido típico de políticas puras (testables sin DB):**

- `Asset::approve()` solo desde `waiting_review`  
- `CoveragePlan::can_run_automatic()` solo si `approved`  
- `FoundPolicy::evaluate(need, candidates) -> Found | Generate`  
- `ExclusionRule::blocks(need) -> bool`  

---

## 5. Application crate — casos de uso

Organizar **por flujo** (igual que producto):

```
application/src/
  lib.rs
  error.rs
  ports/
    repositories.rs
    media_store.rs
    image_provider.rs
    job_queue.rs
    clock.rs
    id_gen.rs
  factory/
    manual_preview.rs
    manual_submit.rs
    automatic_run.rs
  review/
    list_waiting.rs
    approve.rs
    reject.rs
    edit_metadata.rs
    regenerate.rs
    mark_duplicate.rs
  library/
    search.rs
    get.rs
    export_info.rs
    record_usage.rs
  coverage/
    summary.rs
    list_issues.rs
  plans/
    create.rs
    update_items.rs
    approve.rs
    archive.rs
    list.rs
  settings/
    get.rs
    update.rs
  jobs/
    enqueue.rs
    handlers/
      generate_asset.rs
      materialize_plan.rs
      resolve_manual_batch.rs
```

### 5.1 Patrón de use case

```
Input DTO → validate → load aggregates via ports →
domain decisions → persist (tx) → enqueue jobs → Output DTO
```

- Transacciones en el use case (vía port `UnitOfWork` o repositorios transaccionales).  
- Side effects externos (provider HTTP) preferiblemente **dentro de job handlers**, no en el request UI síncrono largo.

### 5.2 Use cases síncronos vs async jobs

| Acción UI | Respuesta síncrona | Job |
|-----------|--------------------|-----|
| Approve | sí | no |
| Preview manual batch | sí (resolve match) | opcional |
| Submit manual generate | acepta batch | `generate_asset` × N |
| Run automatic plan | acepta run | `automatic_plan_materialize` + generates |
| Export grande | acepta | `export_manifest` |

---

## 6. Ports (contratos)

### 6.1 Repositorios

```
ConceptRepository
RepresentationRepository
AssetRepository
GenerationRequestRepository
CoveragePlanRepository
ExclusionRuleRepository
ConceptRelationRepository
ThemeRepository
AssetUsageRepository
SettingsRepository
JobRepository
```

Métodos orientados a casos de uso (no genéricos CRUD ciegos si oscurecen invariantes).

### 6.2 MediaStore

```
allocate_path(asset_id, format) -> RelativePath
write_atomic(tmp, final_path)
read_preview(path)
exists(path)
delete_tmp(job_id)
validate_under_root(path)
```

### 6.3 ImageProvider

```
trait ImageProvider {
  id(&self) -> ProviderRef;
  generate(&self, spec: GenerateSpec) -> Result<GeneratedBytes, ProviderError>;
}
```

MVP: `StubImageProvider` (imagen placeholder o fixture) para desbloquear pipelines sin IA real.

### 6.4 JobQueue / Worker

Ver [06-JOBS.md](./06-JOBS.md).

---

## 7. Infrastructure

```
infrastructure/src/
  sqlite/
    pool.rs
    migrations/
    repos/
  fs_media/
    store.rs
  providers/
    stub.rs
    // real adapters later
  jobs/
    worker.rs
    registry.rs
  config/
    settings.rs
  bootstrap.rs
```

### 7.1 SQLite

- `rusqlite` o `sqlx` (SQLite). **Propuesta:** `rusqlite` + migraciones manuales SQL (simple, desktop-friendly). Decisión en 12.  
- Un `Db` wrapper con `with_tx(|tx| …)`.

### 7.2 Providers reales

Fuera de la fundación. Cuando existan:

- Adapter por provider  
- Secrets vía port `SecretStore`  
- Timeouts y errores mapeados a retryable/terminal  

**No OmniRoute.** Orquestación multi-provider avanzada es non-goal.

---

## 8. Tauri layer (`apps/desktop`)

### 8.1 Composition root

Al arrancar:

1. Load settings / paths  
2. Open DB + migrate  
3. Ensure media dirs  
4. Recover jobs  
5. Start worker  
6. Build `AppState` (handles a services)  
7. Register commands  

### 8.2 Commands

- Un archivo por flujo en `commands/`.  
- Cada command: parse input → call use case → map error → serialize.  
- **Sin lógica de negocio** en commands más allá de mapping.

### 8.3 AppState

```
Db, MediaStore, JobQueue, ProviderRegistry, Settings, Clock
```

Clonado/arc según necesidad Tauri.

### 8.4 Permisos FS

- Scope Tauri limitado al media root y app data.  
- No abrir el home completo sin necesidad.

---

## 9. Errores

Capas de error:

| Capa | Ejemplos |
|------|----------|
| domain | `InvalidAssetTransition`, `PlanNotApproved` |
| application | `NotFound`, `Validation`, `Conflict` |
| infrastructure | `Sqlite`, `Fs`, `Provider` |
| ipc | `code` + `message` + `details` estables para UI |

Códigos estables para el frontend (string enum), no solo texto libre.

---

## 10. Identificadores y tiempo

| Concern | Propuesta |
|---------|-----------|
| IDs | ULID string (orden temporal) |
| Time | Clock port inyectable (tests) |
| Hash | SHA-256 hex de contenido de asset |

---

## 11. Testing backend

| Nivel | Dónde |
|-------|-------|
| Unit domain | `domain` — transitions, found policy |
| Application | use cases con repos in-memory |
| SQLite | tests de integración con temp dir |
| FS media | temp directories |
| Jobs | recovery y idempotencia |
| Commands | smoke invoke con app test harness (fase E2E) |

Detalle en [10-QA.md](./10-QA.md).

---

## 12. Logging y tracing

- `tracing` + subscriber a archivo local en app data.  
- Spans: `use_case`, `job_id`, `asset_id`.  
- No loguear secrets de providers.

---

## 13. Seguridad backend

- Validar enums y JSON payloads.  
- Canonicalizar paths; rechazar `..`.  
- Transacciones para approve y writes relacionados.  
- Rate limit local suave a providers (evitar loops).  

---

## 14. Qué no va en el backend del MVP

- Clientes HTTP a VigilCut  
- GraphQL gateway  
- Microservicios  
- IA multi-agente / OmniRoute  
- Sync engine multi-device  

---

## 15. Mapa flujo → use cases → jobs

| Flujo | Use cases clave | Jobs |
|-------|-----------------|------|
| Manual Factory | preview, submit | resolve (opt), generate_asset |
| Automatic Factory | run_from_plan | materialize, generate_asset |
| Review | approve/reject/… | generate_asset (regen) |
| Library | search/export/usage | export_manifest (opt) |
| Coverage | summary/issues | coverage_recompute (opt) |
| Plans | crud/approve | — |
| Settings | get/update | — |

---

## 16. Criterios de “backend listo para features”

1. Workspace compila.  
2. Migraciones aplican.  
3. Ports + un repo SQLite real (p.ej. settings o concepts).  
4. Job echo durable + recovery test.  
5. Commands bare por flujo responden.  
6. Tests de `Asset::approve` y `Plan::can_run_automatic` verdes.

---

## 17. Referencias

- Architecture: [03-ARCHITECTURE.md](./03-ARCHITECTURE.md)  
- Domain: [02-DOMAIN.md](./02-DOMAIN.md)  
- Jobs: [06-JOBS.md](./06-JOBS.md)  
- Database: [05-DATABASE.md](./05-DATABASE.md)
