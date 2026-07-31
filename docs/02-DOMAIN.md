# 02 — DOMAIN

## 1. Propósito

Definir el **lenguaje ubicuo**, los **agregados**, las **reglas invariantes** y el ciclo de vida de las entidades del MVP.

**No se implementa código aquí.** Solo diseño de dominio.

---

## 2. Lenguaje ubicuo

Usar estos términos de forma consistente en UI, docs, código y SQL.

| Término | Significado | No confundir con |
|---------|-------------|------------------|
| **Theme** | Eje temático de cobertura / crecimiento | Concepto individual |
| **Concept** | Unidad semántica central de la biblioteca | Asset o prompt |
| **Representation** | Forma de expresar un Concept | Estilo visual genérico |
| **Asset** | Materialización (binario + metadata) de una Representation | Concept |
| **GenerationRequest** | Intención de generar un Asset | Job de infraestructura |
| **CoveragePlan** | Plan de **qué** crecer | Ejecución en Factory |
| **CoveragePlanItem** | Unidad planificable dentro de un plan | GenerationRequest (puede originarlo) |
| **AssetUsage** | Registro de uso de un Asset | Export file one-off |
| **ExclusionRule** | Regla que prohíbe o evita ciertas combinaciones | Reject manual |
| **ConceptRelation** | Relación semántica entre Concepts | Carpetas del FS |
| **Job** | Unidad de trabajo durable (infra + orquestación) | Entity de dominio de catálogo |
| **FOUND / GENERATE** | Decisión de reutilización en Manual Factory | Estado de Asset |
| **Waiting Review** | Cola de curación post-generación | Library |

---

## 3. Cadena de valor del dominio

```
Theme
  └── Concept ──┬── Representation ── Asset ── AssetUsage
                └── ConceptRelation
CoveragePlan ── CoveragePlanItem ──► GenerationRequest ──► Asset (waiting_review)
ExclusionRule (cruza Concept / Representation / Style / Provider …)
```

**Idea fuerza:** el catálogo se razona desde **Concept**, no desde el archivo.

---

## 4. Contextos delimitados (bounded contexts)

Para el MVP se reconoce un **Core Domain** con subáreas lógicas (mismos procesos, límites claros de responsabilidad):

| Contexto lógico | Responsabilidad | Flujos |
|-----------------|-----------------|--------|
| **Catalog** | Concepts, Representations, Relations, Themes | Library (lectura), Coverage (lectura) |
| **Acquisition** | GenerationRequests, políticas FOUND/GENERATE | Factory |
| **Curation** | Review decisions, estados de Asset hacia Library | Review |
| **Growth** | Coverage diagnostics, CoveragePlans | Coverage, Plans |
| **Consumption** | AssetUsage, export metadata | Library |
| **Platform** | Jobs, Settings, FS layout, providers config | Settings + infra |

No se fragmenta en microservicios: es un solo proceso Tauri. Los bounded contexts son **módulos de dominio**, no deploys.

---

## 5. Entidades y agregados (diseño)

### 5.1 Theme

**Definición:** agrupación temática usada para cobertura y planes de crecimiento.

| Atributo (conceptual) | Notas |
|-----------------------|-------|
| id | Identidad |
| name | Único legible |
| description | Opcional |
| status | active / archived |
| created_at / updated_at | Auditoría |

**Invariantes:**

- Un Theme no genera assets por sí mismo.  
- Automatic Factory no arranca desde Theme suelto: arranca desde **Plan aprobado** asociado a Theme (u origen de plan).

**Relaciones:**

- Theme 1—N Concept (asignación o membresía; ver decisión de membresía en §10).  
- Theme 1—N CoveragePlan.

---

### 5.2 Concept (agregado raíz del catálogo)

**Definición:** la unidad semántica reutilizable. Todo el producto gira aquí.

| Atributo | Notas |
|----------|-------|
| id | Identidad |
| key / slug | Identificador estable legible (único) |
| name | Nombre de display |
| description | Definición del concepto |
| status | draft / active / deprecated |
| coverage_targets | Objetivos mínimos (p.ej. min representations, min approved assets) — pueden vivir como value objects o settings de coverage |
| created_at / updated_at | |

**Invariantes:**

- Un Concept puede existir sin Assets (hueco de cobertura válido).  
- Deprecar un Concept no borra Assets históricos; afecta planes y recomendaciones.  
- No se navega “CRUD de Concepts” como flujo principal; se crean/actualizan como efecto de Factory/Plans/Review metadata.

**Comportamientos de dominio:**

- Evaluar cobertura (`is_under_covered`, `is_over_covered`) usando targets + conteos de Representations/Assets aprobados.  
- Listar Representations hijas.  
- Aplicar ExclusionRules relevantes.

---

### 5.3 Representation

**Definición:** una forma concreta de expresar un Concept (no el archivo).

| Atributo | Notas |
|----------|-------|
| id | |
| concept_id | Pertenece a un Concept |
| key / name | p.ej. “hero-front”, “detail-macro”, “iconic-simple” |
| description | |
| orientation_default | portrait / landscape / square / any |
| style_hints | value object (no obliga un solo estilo) |
| status | active / deprecated |
| min_approved_assets | target local (opcional; puede heredar del Concept) |

**Invariantes:**

- Representation **siempre** pertenece a exactamente un Concept.  
- Puede haber N Assets por Representation.  
- “Suficientemente bueno” (FOUND) se evalúa a nivel Representation (+ constraints de estilo/orientación/proveedor de la necesidad).

---

### 5.4 Asset

**Definición:** materialización almacenada de una Representation.

| Atributo | Notas |
|----------|-------|
| id | |
| representation_id | |
| concept_id | desnormalizado de lectura (siempre coherente con representation) |
| status | ver ciclo de vida §6 |
| storage_path | ruta relativa al root administrado |
| content_hash | integridad / dedup asistida |
| width / height / mime / format | técnicos |
| orientation | |
| style | |
| provider | origen de generación o import |
| prompt | si aplica |
| generation_request_id | origen, si generada |
| review_notes | |
| duplicate_of_asset_id | si mark duplicate |
| approved_at / rejected_at | |
| created_at / updated_at | |

**Invariantes críticos:**

1. Un Asset **generado** entra en `waiting_review` (nunca `approved` automático).  
2. Solo `approved` es visible en Library.  
3. `rejected` y `duplicate` no son Library.  
4. Regenerar crea un **nuevo** Asset (o nueva revisión ligada), no muta en silencio el binario aprobado.  
5. El binario vive en filesystem administrado; SQLite es source of truth de metadata y estado.

---

### 5.5 GenerationRequest

**Definición:** intención de producir un Asset para una necesidad concreta.

| Atributo | Notas |
|----------|-------|
| id | |
| source | manual_factory / automatic_factory |
| concept_ref | id o key a resolver |
| representation_ref | id o key |
| prompt | |
| orientation | |
| style | |
| provider | |
| decision | pending / found / generate / skipped / failed |
| found_asset_id | si FOUND |
| coverage_plan_item_id | si viene de plan |
| batch_id | agrupación de corrida Manual/Automatic |
| status | draft / queued / running / completed / cancelled |
| result_asset_id | si se generó |
| error | |
| created_at / updated_at | |

**Invariantes:**

- Automatic: `coverage_plan_item_id` **obligatorio** y el plan debe estar **approved**.  
- Manual: lista estructurada; no requiere plan.  
- `decision=found` **no** crea Asset nuevo.  
- `decision=generate` implica Job + eventual Asset en `waiting_review`.  
- GenerationRequest no es lo mismo que Job: el Job ejecuta; el Request expresa la necesidad de dominio.

---

### 5.6 CoveragePlan

**Definición:** decisión de **qué** generar / cubrir. No ejecuta.

| Atributo | Notas |
|----------|-------|
| id | |
| theme_id | opcional pero típico |
| name | |
| description | |
| status | draft / approved / archived / superseded |
| approved_at / approved_by | local user label |
| created_at / updated_at | |

**Invariantes:**

- Solo status `approved` puede alimentar Automatic Factory.  
- Aprobar un plan no genera assets.  
- Archivar / supersede no borra historial de items ejecutados.

---

### 5.7 CoveragePlanItem

**Definición:** unidad de intención dentro de un plan.

| Atributo | Notas |
|----------|-------|
| id | |
| plan_id | |
| concept_ref | |
| representation_ref | opcional si el item es “crear representation” |
| action | ensure_representation / ensure_approved_asset / enrich_variant / … |
| priority | |
| target_count | p.ej. “al menos 1 approved” |
| constraints | style, orientation, provider preferences |
| status | pending / scheduled / fulfilled / cancelled |
| linked_generation_request_ids | trazabilidad |

**Invariantes:**

- Un item no se autoejecuta: Factory lo materializa en GenerationRequests.  
- Cumplir un item se evalúa contra Library (assets approved), no contra waiting_review.

---

### 5.8 AssetUsage

**Definición:** registro de que un Asset fue consumido o referenciado.

| Atributo | Notas |
|----------|-------|
| id | |
| asset_id | debe ser approved al momento de uso idealmente |
| consumer | identificador lógico (p.ej. `vigilcut`, `export-local`) |
| consumer_ref | id externo opcional |
| used_at | |
| context_json | metadata no estructurada mínima |

**Invariantes:**

- No acopla esquema a VigilCut.  
- `consumer` es string estable de catálogo de consumidores, no FK a otra app.  
- Library puede **exportar información** de uso; no implementa el pipeline del consumidor.

---

### 5.9 ExclusionRule

**Definición:** regla que evita combinaciones indeseables en generación o matching FOUND.

| Atributo | Notas |
|----------|-------|
| id | |
| scope | global / theme / concept / representation |
| scope_id | nullable según scope |
| rule_type | forbid_provider / forbid_style / forbid_orientation / forbid_pair / custom |
| payload | value object de la regla |
| active | |
| reason | |

**Invariantes:**

- Se evalúan en Manual/Automatic **antes** de GENERATE.  
- No sustituyen el juicio humano en Review.  
- Una regla no elimina assets ya aprobados; afecta futuro matching/generación.

---

### 5.10 ConceptRelation

**Definición:** vínculo semántico entre Concepts.

| Atributo | Notas |
|----------|-------|
| id | |
| from_concept_id | |
| to_concept_id | |
| relation_type | related / parent_of / contrasts_with / requires / … |
| weight | opcional |
| notes | |

**Invariantes:**

- Sin ciclos en `parent_of` (si se usa jerarquía).  
- No implica herencia automática de assets.  
- Sirve a Coverage (sugerencias) y búsqueda; no es navegación principal.

---

## 6. Ciclo de vida del Asset

```
                    ┌──────────────┐
                    │  (generate)  │
                    └──────┬───────┘
                           ▼
                   waiting_review
                    /    |     \
                   /     |      \
            approved  rejected  duplicate
               │         │         │
               ▼         ▼         ▼
            Library   (hidden)  (hidden; optional link)
               │
               ▼
          AssetUsage (consumption)
```

**Transiciones Review:**

| Desde | Acción | Hacia |
|-------|--------|-------|
| waiting_review | Approve | approved |
| waiting_review | Reject | rejected |
| waiting_review | Mark duplicate | duplicate |
| waiting_review | Edit metadata | waiting_review (mismo, metadata nueva) |
| waiting_review | Regenerate | nuevo Asset waiting_review; el actual puede quedar superseded/rejected según política (§10) |

**Política recomendada (propuesta):** Regenerate deja el asset actual en `superseded` (estado extendido) o `rejected` con razón `regenerated`, y crea uno nuevo en `waiting_review`. Decisión formal en [12-DECISIONS.md](./12-DECISIONS.md).

---

## 7. Ciclo de vida del CoveragePlan

```
draft → approved → (executed via Factory; plan sigue approved)
                 → archived
draft → superseded (si se reemplaza por otro plan)
```

- `approved` es el **único** estado que habilita Automatic Factory.  
- La ejecución no cambia el plan a “running” como entidad de catálogo; el progreso vive en items + jobs + requests.

---

## 8. Decisiones de dominio (FOUND / GENERATE)

**Política “suficientemente bueno” (definición de diseño):**

Un candidato Asset es FOUND para una necesidad si:

1. `status = approved`  
2. Mismo Concept + Representation (resueltos)  
3. Cumple constraints de orientación (exacta o compatible)  
4. Cumple style (exacto o dentro de familia permitida — MVP: exact match o “any”)  
5. No viola ExclusionRule  
6. Provider: match si la necesidad lo exige; si necesidad dice “any”, cualquier provider válido  
7. Preferir mayor calidad / más reciente según ranking configurable en Settings  

Si hay múltiples: seleccionar el mejor según ranking; registrar `found_asset_id`.  
Si cero: GENERATE.

**MVP simplificado de matching:** exact concept + representation + orientation + style (si style no es any). Ranking secundario por `approved_at` desc.

---

## 9. Value objects relevantes

| VO | Uso |
|----|-----|
| Orientation | portrait, landscape, square, any |
| StyleRef | identificador de estilo controlado |
| ProviderRef | proveedor de generación configurado |
| PromptText | texto de generación + opcional negative |
| CoverageTarget | min_representations, min_approved_assets, max_approved_assets |
| StoragePath | path relativo validado dentro del root |
| ContentHash | hash del binario |
| ConsumerId | vigilcut, local-export, … |

Los catálogos de Style/Provider en MVP viven en Settings (config), no como entidades ricas obligatorias.

---

## 10. Reglas invariantes globales (checklist)

1. **No Library sin Approve.**  
2. **Generate → waiting_review** siempre.  
3. **Automatic Factory requiere CoveragePlan approved.**  
4. **Plans no generan binarios.**  
5. **Factory no inventa planes.**  
6. **Concept es el ancla semántica; Asset no sustituye Concept.**  
7. **Jobs durables** respaldan todo trabajo largo (ver 06-JOBS).  
8. **FS administrado** es el único lugar de binarios de Assets.  
9. **Sin dependencia de dominio VigilCut.**  
10. **ExclusionRule afecta matching/generación futura, no reescribe historia aprobada.**  

---

## 11. Eventos de dominio (diseño, no bus externo)

Útiles para auditoría interna y tests de dominio:

| Evento | Cuándo |
|--------|--------|
| ConceptCreated / Deprecated | catálogo |
| RepresentationEnsured | plan o factory |
| GenerationRequested | factory |
| AssetFound | decision FOUND |
| AssetGenerated | binario + metadata created waiting_review |
| AssetApproved / Rejected / Duplicated | review |
| AssetMetadataEdited | review |
| AssetRegenerationRequested | review |
| CoveragePlanApproved | plans |
| CoverageIssueDetected | coverage (derivado) |
| AssetUsed | consumption |

En MVP pueden registrarse como filas de `domain_events` o solo emitirse en capa application para side-effects locales. No hay bus distribuido.

---

## 12. Qué no es dominio de catálogo

| Concepto | Capa |
|----------|------|
| Job state machine de infraestructura | Platform / Jobs |
| Tokens API de proveedores | Settings / secrets locales |
| Layout de ventanas UI | Frontend |
| OmniRoute / IA orquestada avanzada | Non-goal MVP |
| Esquema de proyectos VigilCut | Fuera |

---

## 13. Preguntas de dominio abiertas

Registradas también en decisiones:

1. Membresía Concept↔Theme: N:M o 1:N. **Propuesta:** N:M via `concept_themes`.  
2. Estado `superseded` en Asset al regenerar: ¿sí o se reutiliza rejected? **Propuesta:** `superseded`.  
3. Import de assets existentes (no generados): ¿MVP o post-MVP? **Propuesta:** post-MVP (non-goal inicial), salvo seed de pruebas.  
4. Soft-delete vs archive en Concepts: **Propuesta:** status `deprecated` + archive en Theme/Plan.

---

## 14. Mapa entidad → flujo

| Entidad | Factory | Review | Library | Coverage | Plans | Settings |
|---------|:-------:|:------:|:-------:|:--------:|:-----:|:--------:|
| Theme | R | | R | R | RW | |
| Concept | RW* | R | R | R | R | |
| Representation | RW* | R | R | R | R | |
| Asset | C (waiting) | RW status | R approved | R | | |
| GenerationRequest | CRU | R | | | R | |
| CoveragePlan | R (auto) | | | R | CRU | |
| CoveragePlanItem | R | | | R | CRU | |
| AssetUsage | | | CR | | | |
| ExclusionRule | R | | | R | R | CRU |
| ConceptRelation | | | R | R | R | CRU* |

\* Escrituras de Concept/Representation en Factory/Plans como efecto de asegurar existencia, no como CRUD UI principal.  
\* ConceptRelation: edición avanzada puede vivir en Settings o quedar mínima en MVP.

---

## 15. Siguiente documento

Flujos y transiciones detalladas: [04-WORKFLOWS.md](./04-WORKFLOWS.md)  
Esquema SQLite: [05-DATABASE.md](./05-DATABASE.md)
