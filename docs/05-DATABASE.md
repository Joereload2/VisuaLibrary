# 05 — DATABASE

## 1. Propósito

Diseñar la persistencia **SQLite** de Visual Library: principios, esquema lógico, índices, migraciones y relación con el filesystem.

**No se crean migraciones ni código en esta entrega.**

---

## 2. Principios

1. **SQLite es source of truth** de metadata y estados.
2. **Filesystem es source of truth** de bytes de media.
3. **Una base por instalación/perfil** (archivo local).
4. **Migraciones versionadas** y forward-only en MVP.
5. **IDs opacos** (ULID/UUID text) para entidades de dominio.
6. **Timestamps UTC** ISO-8601 o unix ms (elegir uno y ser consistente; **propuesta:** `TEXT` ISO-8601 UTC).
7. **Enums como TEXT** con check constraints.
8. **No ORM mágico** que oculte SQL en dominio; repos en infrastructure.
9. **Queries de Library siempre filtran `approved`.**
10. **Sin PostgreSQL/Supabase.**

---

## 3. Archivo y pragmas recomendados

```
{app_data}/visual-library/db/visual_library.sqlite
```

Pragmas de diseño (aplicación al abrir):

| Pragma | Valor propuesto | Motivo |
|--------|-----------------|--------|
| `journal_mode` | **WAL** (obligatorio desde el inicio, D-025) | lectores concurrentes UI + worker |
| `foreign_keys` | **ON** (obligatorio, D-025) | integridad |
| `busy_timeout` | 5000 | contención breve |
| `synchronous` | NORMAL | balance desktop |
| `temp_store` | MEMORY | perf |

---

## 4. Tablas del MVP (modelo lógico)

> Nombres en `snake_case`. Campos exactos pueden refinarse en implementación, no el significado.

### 4.1 `schema_migrations`

| Columna | Tipo | Notas |
|---------|------|-------|
| version | TEXT PK | id de migración |
| applied_at | TEXT | |

### 4.2 `themes`

| Columna | Tipo | Notas |
|---------|------|-------|
| id | TEXT PK | |
| name | TEXT NOT NULL UNIQUE | |
| description | TEXT | |
| status | TEXT NOT NULL | active / archived |
| created_at | TEXT NOT NULL | |
| updated_at | TEXT NOT NULL | |

### 4.3 `concepts`

| Columna | Tipo | Notas |
|---------|------|-------|
| id | TEXT PK | |
| key | TEXT NOT NULL UNIQUE | slug estable |
| name | TEXT NOT NULL | |
| description | TEXT | |
| status | TEXT NOT NULL | draft / active / deprecated |
| min_representations | INTEGER NOT NULL DEFAULT 1 | |
| min_approved_assets | INTEGER NOT NULL DEFAULT 1 | |
| max_approved_assets | INTEGER | nullable = sin techo |
| created_at | TEXT NOT NULL | |
| updated_at | TEXT NOT NULL | |

### 4.4 `concept_themes` (N:M propuesta)

| Columna | Tipo | Notas |
|---------|------|-------|
| concept_id | TEXT NOT NULL FK → concepts | |
| theme_id | TEXT NOT NULL FK → themes | |
| PRIMARY KEY (concept_id, theme_id) | | |

### 4.5 `representations`

| Columna | Tipo | Notas |
|---------|------|-------|
| id | TEXT PK | |
| concept_id | TEXT NOT NULL FK → concepts | |
| key | TEXT NOT NULL | único por concept |
| name | TEXT NOT NULL | |
| description | TEXT | |
| orientation_default | TEXT NOT NULL | |
| style_hints | TEXT | JSON |
| status | TEXT NOT NULL | active / deprecated |
| min_approved_assets | INTEGER NOT NULL DEFAULT 1 | |
| created_at | TEXT NOT NULL | |
| updated_at | TEXT NOT NULL | |
| UNIQUE(concept_id, key) | | |

### 4.6 `assets`

| Columna | Tipo | Notas |
|---------|------|-------|
| id | TEXT PK | |
| concept_id | TEXT NOT NULL FK | |
| representation_id | TEXT NOT NULL FK | |
| status | TEXT NOT NULL | waiting_review / approved / rejected / duplicate / superseded |
| storage_path | TEXT NOT NULL | relativo a media root |
| content_hash | TEXT | |
| width | INTEGER | |
| height | INTEGER | |
| mime | TEXT | |
| format | TEXT | |
| orientation | TEXT | |
| style | TEXT | |
| provider | TEXT | |
| prompt | TEXT | |
| generation_request_id | TEXT | FK nullable |
| review_notes | TEXT | |
| reject_reason | TEXT | |
| duplicate_of_asset_id | TEXT | FK self nullable |
| batch_id | TEXT | |
| approved_at | TEXT | |
| rejected_at | TEXT | |
| created_at | TEXT NOT NULL | |
| updated_at | TEXT NOT NULL | |

**Índices:**

- `(status, created_at)` — Review queue
- `(status, concept_id)` — Library / coverage
- `(representation_id, status)` — FOUND matching
- `(content_hash)` — dedup asistida
- `(batch_id)` — factory batches

**Check:** si `status='duplicate'` entonces `duplicate_of_asset_id IS NOT NULL` (deseable).

### 4.7 `generation_requests`

| Columna | Tipo | Notas |
|---------|------|-------|
| id | TEXT PK | |
| source | TEXT NOT NULL | manual_factory / automatic_factory / regenerate |
| batch_id | TEXT | |
| concept_id | TEXT | resuelto |
| representation_id | TEXT | resuelto |
| concept_key | TEXT | entrada original |
| representation_key | TEXT | |
| prompt | TEXT | |
| orientation | TEXT | |
| style | TEXT | |
| provider | TEXT | |
| decision | TEXT | pending / found / generate / skipped / failed |
| found_asset_id | TEXT | |
| coverage_plan_item_id | TEXT | |
| status | TEXT NOT NULL | draft / queued / running / completed / cancelled / failed |
| result_asset_id | TEXT | |
| error | TEXT | |
| created_at | TEXT NOT NULL | |
| updated_at | TEXT NOT NULL | |

**Índices:** `(batch_id)`, `(coverage_plan_item_id)`, `(status)`, `(source, created_at)`.

### 4.8 `coverage_plans`

| Columna | Tipo | Notas |
|---------|------|-------|
| id | TEXT PK | |
| theme_id | TEXT FK nullable | |
| name | TEXT NOT NULL | |
| description | TEXT | |
| status | TEXT NOT NULL | draft / approved / archived / superseded |
| approved_at | TEXT | |
| created_at | TEXT NOT NULL | |
| updated_at | TEXT NOT NULL | |

### 4.9 `coverage_plan_items`

| Columna | Tipo | Notas |
|---------|------|-------|
| id | TEXT PK | |
| plan_id | TEXT NOT NULL FK | |
| concept_id | TEXT | |
| representation_id | TEXT | |
| concept_key | TEXT | |
| representation_key | TEXT | |
| action | TEXT NOT NULL | ensure_representation / ensure_approved_asset / enrich_variant |
| priority | INTEGER NOT NULL DEFAULT 100 | |
| target_count | INTEGER NOT NULL DEFAULT 1 | |
| constraints_json | TEXT | style/orientation/provider |
| status | TEXT NOT NULL | pending / scheduled / fulfilled / cancelled |
| created_at | TEXT NOT NULL | |
| updated_at | TEXT NOT NULL | |

**Índices:** `(plan_id, status, priority)`.

### 4.10 `asset_usages`

| Columna | Tipo | Notas |
|---------|------|-------|
| id | TEXT PK | |
| asset_id | TEXT NOT NULL FK → assets | |
| consumer | TEXT NOT NULL | |
| consumer_ref | TEXT | |
| context_json | TEXT | |
| used_at | TEXT NOT NULL | |

**Índices:** `(asset_id, used_at)`, `(consumer, used_at)`.

### 4.11 `exclusion_rules`

| Columna | Tipo | Notas |
|---------|------|-------|
| id | TEXT PK | |
| scope | TEXT NOT NULL | global / theme / concept / representation |
| scope_id | TEXT | |
| rule_type | TEXT NOT NULL | |
| payload_json | TEXT NOT NULL | |
| active | INTEGER NOT NULL | 0/1 |
| reason | TEXT | |
| created_at | TEXT NOT NULL | |
| updated_at | TEXT NOT NULL | |

### 4.12 `concept_relations`

| Columna | Tipo | Notas |
|---------|------|-------|
| id | TEXT PK | |
| from_concept_id | TEXT NOT NULL FK | |
| to_concept_id | TEXT NOT NULL FK | |
| relation_type | TEXT NOT NULL | |
| weight | REAL | |
| notes | TEXT | |
| created_at | TEXT NOT NULL | |
| UNIQUE(from_concept_id, to_concept_id, relation_type) | | |

### 4.13 `jobs`

Ver detalle completo en [06-JOBS.md](./06-JOBS.md). Resumen:

| Columna | Tipo | Notas |
|---------|------|-------|
| id | TEXT PK | |
| job_type | TEXT NOT NULL | |
| payload_json | TEXT NOT NULL | |
| status | TEXT NOT NULL | queued / running / completed / failed / cancelled / interrupted / waiting_review* |
| priority | INTEGER | |
| attempts | INTEGER | |
| max_attempts | INTEGER | |
| scheduled_at | TEXT | |
| started_at | TEXT | |
| finished_at | TEXT | |
| heartbeat_at | TEXT | |
| last_error | TEXT | |
| related_entity_type | TEXT | generation_request / asset / plan … |
| related_entity_id | TEXT | |
| created_at | TEXT NOT NULL | |
| updated_at | TEXT NOT NULL | |

\* **D-019 (vigente):** jobs de **generación** terminan en `waiting_review` (no `completed`). Jobs sin revisión humana (export, echo, etc.) usan `completed`. Ver `06-JOBS.md` y `ARCHITECTURE.md`.

### 4.14 `job_events` (auditoría ligera)

| Columna | Tipo | Notas |
|---------|------|-------|
| id | TEXT PK | |
| job_id | TEXT NOT NULL FK | |
| at | TEXT NOT NULL | |
| from_status | TEXT | |
| to_status | TEXT | |
| message | TEXT | |

### 4.15 `settings`

| Columna | Tipo | Notas |
|---------|------|-------|
| key | TEXT PK | |
| value_json | TEXT NOT NULL | |
| updated_at | TEXT NOT NULL | |

### 4.16 `domain_events` (opcional MVP+)

| Columna | Tipo | Notas |
|---------|------|-------|
| id | TEXT PK | |
| event_type | TEXT NOT NULL | |
| aggregate_type | TEXT | |
| aggregate_id | TEXT | |
| payload_json | TEXT | |
| created_at | TEXT NOT NULL | |

Puede diferirse a una fase posterior si los tests no lo requieren de inmediato.

### 4.17 `batches` (opcional pero útil)

| Columna | Tipo | Notas |
|---------|------|-------|
| id | TEXT PK | |
| kind | TEXT | manual_factory / automatic_factory |
| label | TEXT | |
| plan_id | TEXT | si auto |
| summary_json | TEXT | counts |
| status | TEXT | |
| created_at | TEXT | |
| finished_at | TEXT | |

---

## 5. Diagrama ER simplificado

```
themes ──< concept_themes >── concepts ──< representations ──< assets
  │             │                 │                              │
  │             │                 ├── concept_relations          ├── asset_usages
  │             │                 │                              │
  └── coverage_plans ──< coverage_plan_items                      │
                              │                                  │
                              └── generation_requests ───────────┘
                              │
jobs (related_entity_*)       exclusion_rules
settings
```

---

## 6. Consultas canónicas (diseño)

### 6.1 Review queue

```sql
SELECT * FROM assets
WHERE status = 'waiting_review'
ORDER BY created_at ASC;
```

### 6.2 Library search (base)

```sql
SELECT a.* FROM assets a
WHERE a.status = 'approved'
  AND (/* filtros dinámicos */)
ORDER BY a.approved_at DESC;
```

### 6.3 FOUND candidate

```sql
SELECT a.* FROM assets a
WHERE a.status = 'approved'
  AND a.representation_id = ?
  AND (a.orientation = ? OR ? = 'any')
  AND (a.style = ? OR ? = 'any')
ORDER BY a.approved_at DESC
LIMIT 1;
```

### 6.4 Concept under-covered

```sql
SELECT c.*
FROM concepts c
WHERE c.status = 'active'
  AND (
    (SELECT COUNT(*) FROM representations r
      WHERE r.concept_id = c.id AND r.status = 'active')
      < c.min_representations
    OR
    (SELECT COUNT(*) FROM assets a
      WHERE a.concept_id = c.id AND a.status = 'approved')
      < c.min_approved_assets
  );
```

### 6.5 Claim next job

```sql
UPDATE jobs
SET status = 'running', started_at = ?, heartbeat_at = ?, attempts = attempts + 1, updated_at = ?
WHERE id = (
  SELECT id FROM jobs
  WHERE status = 'queued' AND (scheduled_at IS NULL OR scheduled_at <= ?)
  ORDER BY priority ASC, created_at ASC
  LIMIT 1
)
RETURNING *;
```

(Patrón exacto puede usar transacción + `BEGIN IMMEDIATE`.)

---

## 7. Integridad referencial y borrado

| Caso | Política MVP |
|------|----------------|
| Borrar Concept con assets | **Prohibido** (solo deprecate) |
| Borrar Asset rejected | Soft: mantener fila; borrado físico FS opcional post-MVP |
| Archivar Theme | No cascada destructiva |
| Plan superseded | Items quedan históricos |

---

## 8. Migraciones

- Directorio: `crates/infrastructure/src/sqlite/migrations/`
- Formato: `0001_init.sql`, `0002_….sql`
- Registro en `schema_migrations`
- Toda migración **revisable** y con test de apply en DB vacía
- **Nunca modificar migraciones ya publicadas** (D-025); solo añadir `000N_….sql` nuevas

**Migración 0001 (alcance conceptual):** tablas del MVP listadas arriba + índices esenciales.

**Duplicados (D-022):** `content_hash` = **SHA-256**. Perceptual hash (pHash) es **post-MVP**.

---

## 9. Filesystem administrado (contrato con DB)

| DB | FS |
|----|----|
| `assets.storage_path` | archivo real bajo media root |
| job tmp paths | `{app_data}/tmp/jobs/{job_id}/` |
| exports | `{app_data}/exports/{export_id}/` |

**Reglas:**

1. Nunca guardar solo path absoluto de usuario sin root configurado.
2. Al borrar/supersede, política de GC de archivos es **fase posterior**.
3. Si falta el archivo pero hay fila approved → issue de integridad (Coverage/health).

---

## 10. Rendimiento (expectativas MVP)

| Escala | Orden de magnitud objetivo |
|--------|----------------------------|
| Concepts | 10³–10⁴ |
| Assets | 10⁴–10⁵ |
| Jobs/día | cientos–miles |

Índices anteriores bastan; FTS5 para search de prompts/nombres es **fase posterior** si hace falta.

---

## 11. Backup / portabilidad

- Cerrar o checkpoint WAL antes de copiar.
- Backup = copiar `visual_library.sqlite*` + árbol `media/`.
- Documentar en Settings (fase docs in-app posterior).

---

## 12. Seguridad SQL

- 100% queries parametrizadas.
- Sin concatenar input de UI en SQL.
- JSON en columnas TEXT validado en application layer.

---

## 13. No-goals de base de datos

- Replicación multi-dispositivo
- Sync cloud
- Sharding
- Postgres compatibility layer
- Guardar blobs BLOB de imágenes en SQLite

---

## 14. Referencias

- Jobs: [06-JOBS.md](./06-JOBS.md)
- Dominio: [02-DOMAIN.md](./02-DOMAIN.md)
- Backend repos: [08-BACKEND.md](./08-BACKEND.md)
