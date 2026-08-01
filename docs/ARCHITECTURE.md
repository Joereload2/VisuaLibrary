# ARCHITECTURE — Visual Library

**Autoridad de arquitectura (Foundation 0).**
`docs/03-ARCHITECTURE.md` es referencia ampliada; si hay conflicto, **gana este archivo**.
Sin implementación en este documento.

---

## 1. Resumen

| Decisión | Valor real / objetivo |
|----------|------------------------|
| Tipo | Desktop **local-first** |
| Shell | **Tauri 2** (`apps/desktop`) |
| Core | **Rust** workspace |
| UI | **React 19 + Vite 6 + TypeScript** (`packages/ui`) |
| Metadata | **SQLite** (WAL + FKs; migraciones 0001/0002) |
| Bytes | **Filesystem administrado** bajo `media_root` |
| Jobs | Tabla `jobs` + generate stub **en proceso**; recovery de `running` al boot |
| Package manager JS | **pnpm** 8 workspace |
| Nube / Postgres / Supabase | **No** en el núcleo |

---

## 2. Estructura del monorepo (real)

```text
VisuaLibrary/
  apps/desktop/              # @visual-library/desktop + src-tauri
  packages/ui/               # @visual-library/ui (React, Vitest, Playwright)
  crates/domain/             # visual_library_domain (puro)
  crates/application/        # visual_library_application (use cases + ports)
  crates/infrastructure/     # visual_library_infrastructure (adapters)
  docs/                      # producto + playbook + constituciones
```

Estado de código: **F1–F6 MVP usable** — 6 estaciones con flujos reales (catálogo, generate stub, Review, Library, Factory manual/automatic, Plans, Coverage). Post-MVP: search/export, worker completo, providers IA.

---

## 3. Capas y dependencias

```text
UI (packages/ui)
    │  invoke / IPC
    ▼
API adapters (apps/desktop/src-tauri commands)
    │
    ▼
application  ──►  domain
    │
    ▼
infrastructure: persistence | filesystem | providers | jobs
```

| Regla |
|-------|
| `domain` no depende de Tauri, SQLite, FS, HTTP, env |
| `application` depende de `domain` + ports (traits) |
| `infrastructure` implementa ports |
| Commands Tauri = adapters delgados (sin reglas de negocio largas) |
| UI no es source of truth |

---

## 4. Módulos de aplicación (por flujo)

Alineados al producto, no a tablas:

| Módulo | Responsabilidad |
|--------|-----------------|
| factory | Manual + Automatic; FOUND/GENERATE; encolar generate |
| review | Cola waiting_review; approve/reject/… |
| library | Search/export solo approved; usage |
| coverage | Issues accionables |
| plans | Draft/approve planes e items |
| settings | Paths, providers config, umbrales |
| jobs | Cola durable, recovery, cancel/retry |

---

## 5. Dominio (frontera conceptual)

Cadena:

```text
Theme → Concept → Representation → Asset → AssetUsage
CoveragePlan → CoveragePlanItem → GenerationRequest → Asset (waiting_review)
ExclusionRule · ConceptRelation
```

**Invariantes de frontera:**

| # | Invariante |
|---|------------|
| 1 | Job de **generación** termina en **`waiting_review`** (no `completed`) |
| 2 | Asset generado entra `waiting_review`; solo **Approve** → Library |
| 3 | Automatic requiere CoveragePlan **approved** |
| 4 | Plans no generan binarios ni llaman providers |
| 5 | Library queries solo `approved` |
| 6 | Sin tipos/esquemas VigilCut en el core |

Detalle de entidades: `docs/02-DOMAIN.md` (referencia).

---

## 6. Persistencia (contratos)

| Store | Dueño de |
|-------|----------|
| SQLite | metadata, estados, jobs, planes, settings keys (no secrets) |
| FS administrado | bytes de media, tmp de jobs, exports |

Normas (al implementar):

- WAL desde el inicio
- `foreign_keys=ON`
- migraciones numeradas; **no editar publicadas**
- paths relativos al media root; anti path-traversal
- hash de contenido **SHA-256** (pHash post-MVP)

---

## 7. Jobs (contratos)

- Persistir job **antes** de ejecutar
- Worker en proceso (no microservicio)
- Generate → status **`waiting_review`** + Asset `waiting_review`
- Approve/Reject = transiciones posteriores de Asset
- Cancelación cooperativa; cleanup solo tmp propio
- `idempotency_key` para reintentos seguros
- Progreso persistido (eventos no son la verdad)

---

## 8. API (Tauri commands)

| Preferir | Evitar |
|----------|--------|
| `approve_asset`, `search_library`, `submit_manual_batch` | `set_status`, `update_row`, `run_sql` |

Errores estructurados hacia UI:

```text
code, message, retryable, suggested_action, detail?
```

Hoy existe solo: `health`.

---

## 9. Frontend (fronteras)

- Organización `packages/ui/src/flows/{factory,review,library,coverage,plans,settings}`
- Rutas primarias = 6 estaciones
- Estado efímero en React; datos/jobs desde IPC snapshots
- Tests: Vitest + Testing Library; E2E Playwright **sobre Vite** (no Tauri completo aún)

---

## 10. Providers

```text
port ImageProvider → Stub (MVP) → adapter real (fase posterior)
```

- Dominio no conoce OmniRoute ni un vendor concreto
- Secrets del provider real: **OS secure store** (nunca SQLite/JSON/plano/logs)

---

## 11. Fronteras externas

| Externo | Relación |
|---------|----------|
| VigilCut | Consumidor futuro vía export/usage; **sin acoplamiento** |
| Image providers | Adapters opcionales configurados en Settings |
| Cloud DB | Fuera de alcance del núcleo |

---

## 12. Runtime de datos (diseño)

```text
{app_data}/visual-library/
  db/visual_library.sqlite
  media/assets/...
  exports/
  logs/
  tmp/jobs/{job_id}/
```

---

## 13. Qué no es esta arquitectura

- Microservicios
- Electron como dirección
- SQL desde el frontend
- UI-first feature development
- Bus de eventos distribuido

---

## 14. Referencias

- Playbook: `docs/AI_PLAYBOOK.md`
- Producto: `docs/PRODUCT.md`
- Ingeniería: `docs/constitution/ENGINEERING.md`
- Decisiones: `docs/12-DECISIONS.md`
