# 07 — FRONTEND

## 1. Propósito

Definir la organización del frontend de Visual Library: principios, estructura por **flujos**, contratos con Tauri y límites de responsabilidad.

**No se diseñan pantallas visuales ni se escribe UI de negocio.**  
Ver [09-UX.md](./09-UX.md) para estaciones y navegación.

---

## 2. Rol del frontend

El frontend es un **cliente de presentación** de los seis flujos.

| Hace | No hace |
|------|---------|
| Navegación por estaciones | SQL directo |
| Formularios y validación de UX | Invariantes de dominio definitivas |
| Previews y estados de carga | Aprobar “en localStorage” |
| Llamadas IPC a commands | Ejecutar generación real |
| Optimistic UI solo si el backend confirma | Ser source of truth |

Toda verdad de negocio se confirma en Rust/SQLite.

---

## 3. Stack propuesto

| Pieza | Propuesta de fundación |
|-------|------------------------|
| Lenguaje | TypeScript (strict) |
| UI runtime | WebView Tauri 2 |
| Framework | **React** (amplio ecosistema Tauri) — alternativa aceptable: Svelte; **decisión formal en 12** |
| Routing | Router por flujos (6 rutas raíz) |
| Data fetching | Capa `api/` que envuelve `invoke` |
| Estilos | Sistema mínimo propio (tokens); sin inventar brand completo aún |
| Test UI | Component tests + E2E (Playwright/WebDriver Tauri en fases QA) |

La elección exacta del framework se fija en Fase 1 scaffold; no bloquea el resto del diseño.

---

## 4. Principio de organización: por flujos, no por tablas

### 4.1 Prohibido como navegación de primer nivel

- Conceptos  
- Representaciones  
- Assets  
- GenerationRequests  
- Tablas del ER  

### 4.2 Rutas de primer nivel (MVP)

```
/factory
/factory/manual
/factory/automatic
/review
/library
/coverage
/plans
/settings
```

Rutas de detalle **anidadas al flujo**, no al modelo:

```
/review/:assetId
/library/:assetId
/plans/:planId
/factory/batches/:batchId
```

---

## 5. Estructura de paquetes UI (propuesta)

```
packages/ui/
  src/
    main.tsx
    app/
      AppShell.tsx          # chrome: nav de 6 flujos
      router.tsx
      providers.tsx
    flows/
      factory/
        ManualFactoryView.tsx      # placeholder en fases
        AutomaticFactoryView.tsx
        api.ts
        types.ts
        model/                     # view-models, no domain rust
      review/
      library/
      coverage/
      plans/
      settings/
    shared/
      ipc/
        client.ts
        errors.ts
      ui/                          # botones, layout primitives
      hooks/
    assets/                        # iconos estáticos de app
```

Cada `flows/<name>/api.ts` solo conoce commands de ese flujo (+ jobs si aplica).

---

## 6. Capa IPC

### 6.1 Reglas

1. Un módulo `invoke` tipado; no esparcir strings mágicos.  
2. DTOs del frontend **no** son entidades de dominio completas; son contracts.  
3. Errores: mapear a mensajes de estación (Factory vs Settings).  
4. Listados: paginación (`cursor` o `offset` + `limit`).  
5. No filtrar Library en cliente omitiendo `status` — el backend solo devuelve approved en search Library.

### 6.2 Grupos de API frontend

| Módulo | Commands (ilustrativos) |
|--------|-------------------------|
| `flows/factory/api` | previewManualBatch, submitManualBatch, runAutomaticPlan, listBatches |
| `flows/review/api` | listWaiting, approve, reject, editMetadata, regenerate, markDuplicate |
| `flows/library/api` | search, get, exportInfo, recordUsage |
| `flows/coverage/api` | summary, listIssues |
| `flows/plans/api` | list, get, create, updateItems, approve, archive |
| `flows/settings/api` | get, update, validatePaths |
| `shared/jobs/api` | listJobs, cancelJob, retryJob |

---

## 7. Estado de UI

| Tipo de estado | Dónde |
|----------------|-------|
| Server state (listas, detalles) | cache de queries por flujo |
| Form drafts (plan draft, manual list) | estado local del flujo; persistencia backend al submit |
| Session UI (tab activo) | router + minimal store |
| Job progress | polling o event Tauri `job_updated` |

**No** guardar colas de generación solo en estado React.

---

## 8. Eventos desde backend (diseño)

Opcional pero útil:

- `job://updated`  
- `review://queue_changed`  
- `library://asset_approved`  

El frontend se suscribe en el shell y invalida caches del flujo afectado.

---

## 9. Responsabilidades por flujo (frontend)

### Factory

- Editor/import de lista manual (estructura, no “prompt playground” libre sin schema).  
- Selección de plan aprobado (automatic).  
- Resumen FOUND/GENERATE.  
- Progreso de batch/jobs.  
- **No** galería de Library completa.

### Review

- Cola y detalle de waiting_review.  
- Acciones Approve/Reject/Edit/Regenerate/Duplicate.  
- **No** búsqueda general de approved como tarea principal.

### Library

- Search/filter/detail/export.  
- **No** botones de generar/aprobar.

### Coverage

- Summary + lista de issues accionables.  
- CTAs a Plans / Review / Library.  
- **No** charts sin tabla de problemas.

### Plans

- Drafting de plan e items.  
- Approve plan.  
- Progreso de items.  
- **No** trigger oculto de providers.

### Settings

- Paths, providers, thresholds, job concurrency.  
- **No** operaciones de producción.

---

## 10. Accesibilidad y UX técnica (mínimos)

- Navegación por teclado en colas Review.  
- Confirmación en acciones destructivas (reject masivo, cancel jobs).  
- Estados empty/loading/error por estación.  
- No bloquear UI thread con binarios: previews vía paths convertidos por Tauri asset protocol / commands de thumbnail (fase media).

---

## 11. Internacionalización

- MVP puede ser **es-ES** first (producto documentado en español).  
- Claves i18n desde el inicio si el costo es bajo; si no, strings centralizados en un módulo.

---

## 12. Testing frontend

| Nivel | Qué |
|-------|-----|
| Unit | pure functions de view-model, parsers de CSV/lista manual |
| Component | acciones de Review llaman api mocks |
| E2E | ver [10-QA.md](./10-QA.md) |

No testear invariantes de dominio en el frontend como única red.

---

## 13. Límites de fase

| Fase temprana | El frontend muestra |
|---------------|---------------------|
| Scaffold | Shell + 6 rutas empty |
| Domain/DB | Read models vacíos |
| Review stub | Lista fake → luego real |
| Factory | Provider stub messages |

**Prohibido:** inventar pantallas extra “bonitas” fuera de los 6 flujos.

---

## 14. Referencias

- UX: [09-UX.md](./09-UX.md)  
- Architecture: [03-ARCHITECTURE.md](./03-ARCHITECTURE.md)  
- Backend commands: [08-BACKEND.md](./08-BACKEND.md)
