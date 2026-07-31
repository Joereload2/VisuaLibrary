# 04 — WORKFLOWS

## 1. Propósito

Especificar los **seis flujos del MVP**, sus entradas/salidas, reglas de transición y fronteras.

No se diseñan pantallas visuales aquí (ver [09-UX.md](./09-UX.md) para estaciones).
No se implementa lógica.

---

## 2. Mapa global

```
                 ┌──────── Settings ────────┐
                 │                          │
                 ▼                          ▼
   Theme/Config                    Providers, paths, thresholds
                 │
                 ▼
              ┌──────┐
              │Plans │  decide QUÉ
              └──┬───┘
                 │ approved plan
                 ▼
         ┌───────────────┐
         │   Factory     │  decide CÓMO
         │ Manual | Auto │
         └───────┬───────┘
                 │ new assets
                 ▼
         ┌───────────────┐
         │    Review     │
         └───────┬───────┘
                 │ approve
                 ▼
         ┌───────────────┐
         │   Library     │
         └───────┬───────┘
                 │ metrics
                 ▼
         ┌───────────────┐
         │   Coverage    │ ──► sugiere ──► Plans
         └───────────────┘
```

**Regla de oro:** ninguna flecha salta Review hacia Library.

---

## 3. Flujo 1 — Factory

### 3.1 Objetivo

Crear (o reutilizar) material visual para necesidades explícitas, sin contaminar Library.

### 3.2 Subflujo A — Manual Factory

#### Propósito

Producción controlada a partir de una **lista estructurada de necesidades**.

#### Entrada

Lista de ítems, cada uno con:

| Campo | Requerido | Descripción |
|-------|:---------:|-------------|
| concepto | sí | key o nombre resoluble |
| representación | sí | key o nombre resoluble bajo el concepto |
| prompt | sí si GENERATE probable | texto de generación |
| orientación | sí | portrait / landscape / square / any |
| estilo | sí | style ref o any |
| proveedor | sí | provider ref o any |

Campos opcionales MVP: notas, prioridad, id externo de lote.

#### Pasos

1. **Ingest** de la lista (validación de schema).
2. **Resolve** Concept y Representation (crear si política lo permite; MVP: fail si no existen **o** auto-ensure mínimo — ver decisiones; **propuesta:** auto-ensure Concept/Representation en draft si missing, flaggeado).
3. **Match** de Asset aprobado “suficientemente bueno” (política dominio).
4. **Decide** por ítem:
   - `FOUND` + `found_asset_id`
   - `GENERATE`
   - `SKIPPED` (exclusion rule / invalid)
5. **Enqueue** GenerationRequests + Jobs solo para `GENERATE`.
6. **Execute** generación (provider adapter).
7. **Persist** Asset en `waiting_review` + binario en FS.
8. **Report** batch summary: found / generate / failed / skipped.

#### Salidas

| Resultado | Destino |
|-----------|---------|
| FOUND | Referencia a Asset approved existente (no Review) |
| GENERATE ok | Asset nuevo → **Review** |
| GENERATE fail | Request/Job failed; sin Asset library |
| SKIPPED | Sin Asset nuevo |

#### Prohibiciones

- Aprobar automáticamente.
- Escribir en Library.
- Generar ítems no presentes en la lista.
- Ignorar ExclusionRules.

#### Jobs típicos

- `manual_batch_resolve`
- `generate_asset` (N veces)

---

### 3.3 Subflujo B — Automatic Factory

#### Propósito

Crecimiento de cobertura a partir de un **Coverage Plan aprobado**.

#### Entrada canónica

```
Theme → CoveragePlan(approved) → CoveragePlanItems
     → GenerationRequests → Jobs → Assets(waiting_review)
```

#### Pasos

1. Seleccionar `CoveragePlan` en estado `approved`.
2. Cargar items `pending` (o `scheduled`) ordenados por priority.
3. Para cada item, materializar una o más `GenerationRequest` según `action` y `target_count`.
4. Evaluar FOUND vs GENERATE igual que Manual (reutilización primero).
5. Encolar solo GENERATE.
6. Persistir assets en `waiting_review`.
7. Actualizar progreso de items (fulfilled solo si targets de **approved** se cumplen — puede requerir Review posterior; ver nota).

#### Nota de cumplimiento de items

Un item con `ensure_approved_asset` **no** se marca `fulfilled` al generar; se marca cuando Library tiene el conteo approved requerido.
Estados intermedios del item: `scheduled` mientras hay requests/jobs en vuelo o assets en review.

#### Prohibiciones

- Ejecutar plan `draft` / `archived`.
- Generación aleatoria sin items.
- Mezclar “inventar conceptos libres” fuera del plan.
- Saltar Review.

---

## 4. Flujo 2 — Review

### 4.1 Objetivo

Control de calidad humano de todo material nuevo.

### 4.2 Entrada

Cola de Assets con `status = waiting_review` (y opcionalmente filtros por batch, concepto, provider).

### 4.3 Acciones

| Acción | Precondiciones | Efecto |
|--------|-----------------|--------|
| **Approve** | waiting_review | → `approved`; `approved_at`; visible en Library |
| **Reject** | waiting_review | → `rejected`; razón opcional; no Library |
| **Edit metadata** | waiting_review | actualiza metadata; permanece waiting_review |
| **Regenerate** | waiting_review | política de supersede + nuevo GenerationRequest/Job + nuevo Asset waiting_review |
| **Mark duplicate** | waiting_review | → `duplicate`; `duplicate_of_asset_id` requerido o sugerido |

### 4.4 Salidas

- Library crece solo por Approve.
- Coverage se actualiza al cambiar conteos approved.
- Plan items pueden pasar a fulfilled tras Approve.

### 4.5 Prohibiciones

- Generar desde Review **excepto** Regenerate (que es re-acquire controlado).
- Editar binarios aprobados in-place sin nuevo ciclo (post-MVP si hubiera “replace”).
- Approve masivo sin listar (el producto puede permitir bulk approve **explícito** más adelante; MVP: al menos confirmación por selección consciente).

### 4.6 Jobs

- `generate_asset` (desde regenerate)
- no se requiere job para approve/reject (transacción corta)

---

## 5. Flujo 3 — Library

### 5.1 Objetivo

Consultar el catálogo confiable.

### 5.2 Entrada

Queries de usuario: texto, concept, representation, theme, orientation, style, provider, tags futuros.

### 5.3 Operaciones

| Operación | Descripción |
|-----------|-------------|
| Search / Filter | Solo `approved` |
| Consultar detalle | Metadata + preview path seguro |
| Exportar información | JSON/CSV/manifest de selección (metadata + paths relativos o copies controladas) |
| Registrar uso | `AssetUsage` opcional al exportar |

### 5.4 Prohibiciones

- Generar imágenes.
- Review.
- Mostrar waiting_review / rejected / duplicate como catálogo principal.
- Mutar Concept de forma libre (solo lectura de catálogo aquí).

---

## 6. Flujo 4 — Coverage

### 6.1 Objetivo

Diagnosticar la salud de la biblioteca en términos **conceptuales** y producir **problemas accionables**.

### 6.2 Preguntas canónicas

1. ¿Cuántos conceptos existen (active)?
2. ¿Cuáles están mal cubiertos (bajo min targets)?
3. ¿Cuáles tienen demasiadas imágenes (sobre max)?
4. ¿Cuáles no tienen suficientes representaciones?
5. ¿Qué themes tienen huecos?
6. ¿Cuántos assets están atascados en Waiting Review? (operativo)

### 6.3 Tipos de issue (diseño)

| Code | Severidad tipica | Acción sugerida |
|------|------------------|-----------------|
| `concept_under_covered` | high | Crear Plan items |
| `representation_missing` | high | Plan: ensure_representation |
| `representation_under_covered` | medium | Plan: ensure_approved_asset |
| `concept_over_covered` | low | Exclusion / stop generating |
| `theme_gap` | medium | Nuevo plan por theme |
| `review_backlog` | medium | Ir a Review |
| `orphan_waiting` | low | Review o cancel jobs |

### 6.4 Salidas

- Summary counts
- Lista de issues accionables (con deep links lógicos a Plans / Review / Library)
- **No** solo gráficos vacíos de acción

### 6.5 Prohibiciones

- Ejecutar generación desde Coverage.
- Aprobar assets.
- Ser un explorador CRUD de tablas.

---

## 7. Flujo 5 — Plans

### 7.1 Objetivo

Decidir **qué** debe generarse o asegurarse en el catálogo.

### 7.2 Operaciones

| Operación | Descripción |
|-----------|-------------|
| Crear plan | draft + theme opcional |
| Añadir/editar/quitar items | actions + targets + constraints |
| Aprobar plan | draft → approved (habilita Automatic Factory) |
| Archivar / supersede | cierra plan |
| Consultar progreso | items pending/scheduled/fulfilled |

### 7.3 Relación con Factory

```
Plans.approve  ≠  generate
AutomaticFactory.run(plan_id)  ⇒  requests + jobs
```

### 7.4 Prohibiciones

- Llamar providers de imagen.
- Escribir assets.
- Confundir UI de plan con batch manual (entradas distintas).

---

## 8. Flujo 6 — Settings

### 8.1 Objetivo

Configurar el entorno local.

### 8.2 Áreas de configuración (MVP)

| Área | Ejemplos |
|------|----------|
| Paths | media root, export default |
| Providers | enable/disable, credentials ref, defaults |
| Coverage defaults | min representations, min/max assets |
| Matching | política FOUND (exactitud style/orientation) |
| Jobs | concurrency, retry limits |
| UI prefs | idioma, densidad (mínimo) |

### 8.3 Prohibiciones

- Producción (generar, aprobar, planificar ejecución).
- Convertirse en admin de todas las entidades del dominio.

---

## 9. Workflows transversales

### 9.1 Arranque de aplicación

1. Cargar settings.
2. Abrir SQLite; aplicar migraciones.
3. Verificar media root.
4. Recover jobs: `running` → `interrupted` → requeue o failed según política.
5. Iniciar worker.
6. UI en estación default (Library o Coverage summary — UX).

### 9.2 Exportación

1. Solo desde Library (approved).
2. Generar manifest + opcionalmente copiar binarios a carpeta export.
3. Registrar AssetUsage si el usuario lo confirma.

### 9.3 Cancelación

- Usuario cancela job `queued`/`running` → `cancelled`.
- No borra assets ya escritos en waiting_review sin acción Review.

---

## 10. Máquinas de estado resumidas

### Asset

`waiting_review` → `approved` | `rejected` | `duplicate` | `superseded`

### GenerationRequest

`draft` → `queued` → `running` → `completed` | `failed` | `cancelled`

### Job (infra)

`queued` → `running` → `waiting_review` | `completed` | `failed` | `cancelled` | `interrupted`

- **Generate:** terminal = **`waiting_review`** (D-019; job y asset).
- **Sin review humano** (export, echo, materialize, …): terminal = `completed`.
- Detalle: [06-JOBS.md](./06-JOBS.md).

### CoveragePlan

`draft` → `approved` → `archived` | `superseded`

### CoveragePlanItem

`pending` → `scheduled` → `fulfilled` | `cancelled`

---

## 11. Matriz de permisos lógicos (single user)

En MVP no hay RBAC; todas las estaciones están disponibles.
La matriz sirve para no mezclar side-effects:

| Acción | Factory | Review | Library | Coverage | Plans | Settings |
|--------|:-------:|:------:|:-------:|:--------:|:-----:|:--------:|
| Generar binario | ✓ | regen only | | | | |
| Approve asset | | ✓ | | | | |
| Ver approved | ✓(found) | ✓ | ✓ | ✓ | ✓ | |
| Editar plan | | | | | ✓ | |
| Cambiar media root | | | | | | ✓ |

---

## 12. Criterios de aceptación transversales de flujos

1. Dado un GENERATE exitoso, el Asset aparece en Review y no en Library.
2. Dado Approve, aparece en Library y deja de estar en Waiting Review.
3. Dado Automatic Factory con plan draft, la operación falla.
4. Dado Manual con match exacto approved, decision=FOUND y cero jobs generate.
5. Dado Coverage issue under-covered, existe path lógico a crear Plan item.
6. Dado reinicio de app con job running, el sistema no pierde el job (interrupted/recover).

---

## 13. Referencias

- Dominio: [02-DOMAIN.md](./02-DOMAIN.md)
- Jobs: [06-JOBS.md](./06-JOBS.md)
- UX estaciones: [09-UX.md](./09-UX.md)
