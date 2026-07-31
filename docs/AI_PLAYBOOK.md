# AI Playbook — Visual Library

**Autoridad:** metodología de desarrollo
**Normativo.** Los prompts deben ser cortos; las reglas viven aquí.

---

## 1. Filosofía

Construir **software** (verificable, reversible, documentado), no código suelto.

| Principio | Implicación |
|-----------|-------------|
| Pequeño | Una tarea = un incremento acotado |
| Verificable | Criterios de aceptación + pruebas del riesgo |
| Documentado | Cambios de producto/arquitectura → ADR o doc |
| Local-first | Núcleo offline |
| Flujos, no tablas | Seis estaciones; sin menú de entidades |
| Supervisable | Jobs automáticos; humanos en Review |

**No asumas** en producto, dominio o arquitectura → STOP.

---

## 2. Autoridad documental (única)

| Tema | Fuente de verdad |
|------|------------------|
| Metodología | **Este playbook** |
| Producto / MVP | `PRODUCT.md` |
| Arquitectura | `ARCHITECTURE.md` |
| Ingeniería | `constitution/ENGINEERING.md` |
| UX/UI | `constitution/UX_UI.md` |
| Testing | `constitution/TESTING.md` |
| Seguridad | `constitution/SECURITY.md` |
| Done | `constitution/DONE.md` |
| Decisiones puntuales | `12-DECISIONS.md` |

Todo lo demás es **referencia** o **legado** (ver `00-START-HERE.md`).
Conflicto → gana la tabla de arriba + ADR aceptados.

### Lectura mínima por tarea

| Tarea | Leer |
|-------|------|
| Cualquiera | Este playbook + docs de las **capas que tocas** |
| Docs only / LOW | Playbook (skimming) + archivo que editas |
| Feature con UI | + PRODUCT (flujos) + UX_UI + ENGINEERING + TESTING |
| Persistencia/jobs | + ARCHITECTURE + ENGINEERING + SECURITY |
| Cierre | + TESTING + DONE |

**No** releer todo el repo en cada tarea (escala a cientos de pantallas).

---

## 3. Definition of Ready

### Completo (MEDIUM o superior, o cualquier feature de producto)

Antes de codificar, debe existir:

1. Objetivo (una frase medible)
2. Alcance / no-objetivos
3. Criterios de aceptación verificables
4. Riesgo (LOW | MEDIUM | HIGH | ARCHITECTURE)
5. Capas afectadas (lista corta)

Si falta 1–3 o 4–5 en feature de producto → **STOP**.

### Ligero (LOW: docs, copy, test de helper, fix tipográfico)

Basta con: objetivo + archivo(s) + “no cambia producto/arquitectura”.

### Task Card

```text
## TASK CARD
Objetivo:
Alcance / No objetivos:
Riesgo: LOW | MEDIUM | HIGH | ARCHITECTURE
Capas: domain | persistence | jobs | backend | api | frontend | docs | …
Criterios de aceptación:
- [ ] …
Pruebas previstas: (según TESTING.md)
```

---

## 4. STOP RULES

Parar y pedir aprobación si:

| # | Condición |
|---|-----------|
| 1 | Cambia dominio, MVP o arquitectura documentada |
| 2 | Contradice una fuente de verdad (§2) |
| 3 | Requiere **dependencia nueva** |
| 4 | Migración destructiva o editar migración **publicada** |
| 5 | Toca **≥3** de: domain, persistence, jobs, backend, api, frontend **sin** poder dividir |
| 6 | No hay criterios de aceptación (si no es LOW ligero) |
| 7 | Duda de producto material |
| 8 | Pedirían saltar Review / Library gate / plan approved |
| 9 | Presentar placeholder como funcionalidad Done |

---

## 5. Riesgo

| Riesgo | Ejemplo | Pruebas | Revisión multi-rol |
|--------|---------|---------|---------------------|
| **LOW** | docs, helper, estilo | Unit del cambio si hay código; `git diff --check` | No |
| **MEDIUM** | un use case o un flujo UI parcial | Unit + integration si I/O; build/check | Auto-check capas tocadas |
| **HIGH** | Factory, Review, jobs, matching | + E2E flujo + regression si bugfix | Sí (PM/PO/QA + capas) |
| **ARCHITECTURE** | monorepo, migraciones base, IPC, secrets | Todo lo aplicable + ADR | Sí (incluye Arquitectura) |

Detalle de comandos: `constitution/TESTING.md`.

---

## 6. Orden de implementación (features de producto)

```text
Producto → Dominio → Persistencia → Jobs → Backend → API → Frontend → UX polish → QA → Refactor
```

- No saltar etapas sin contratos/aceptación de la anterior.
- **UI nunca es la primera capa** de una feature nueva de negocio.
- El scaffold de 6 rutas ya existe; no inventar pantallas de negocio antes de dominio/API del flujo.

Tareas LOW de docs/infra no siguen este pipeline completo.

---

## 7. Capas (minimizar)

| Capa | Repo |
|------|------|
| domain | `crates/domain` |
| persistence | `crates/infrastructure` (SQLite/FS) |
| jobs | worker en infrastructure |
| backend | `crates/application` |
| api | `apps/desktop/src-tauri` |
| frontend | `packages/ui` |

Preferir **una o dos** capas por PR. Si hace falta un vertical slice, dividir en PRs ordenados por §6.

---

## 8. Ciclo de trabajo

```text
1. TASK CARD + DoR
2. STOP? → parar
3. Implementar (capas mínimas, orden §6 si feature)
4. Pruebas del riesgo (TESTING.md)
5. git diff --check
6. Loop errores → corregir
7. Loop simplificar (sin rediseñar ni ampliar alcance)
8. Si HIGH/ARCHITECTURE: revisión multi-rol breve
9. Entrega → sin commit/push salvo orden → esperar OK
```

### Revisión multi-rol (solo HIGH / ARCHITECTURE)

Para cada rol **aplicable**: ¿aprobaría? (sí/no) + un problema si no.

Roles: PM · PO · UX (si UI) · FE · BE · QA · Arquitectura (si ARCHITECTURE) · Security (si secrets/FS/red).

LOW/MEDIUM: basta checklist DONE aplicable + pruebas.

---

## 9. Entrega

```text
## ENTREGA
Resumen:
Archivos:
Pruebas ejecutadas / omitidas (motivo):
Riesgos:
Capturas: (solo UI real de feature)
git diff --check: OK|FAIL
Commit/Push: NO
```

---

## 10. Comandos reales (no inventar)

```text
pnpm install
pnpm test:e2e:install   # una vez por máquina

pnpm fmt:rust           # si tocó Rust
pnpm check:rust
pnpm test:rust

pnpm test:ui            # si tocó UI
pnpm build:ui
pnpm test:e2e           # HIGH+ con UI de flujo (cuando webServer OK)

pnpm test / pnpm quality
pnpm dev / pnpm dev:ui
git diff --check
```

**Huecos:** ESLint, Prettier, CI, clippy-as-CI-gate, E2E webServer a veces timeout en `127.0.0.1:1420`.

---

## 11. Invariantes de producto (no romper)

1. Job generate → **`waiting_review`** (no `completed`)
2. Library solo con **Approve**
3. Automatic solo plan **approved**
4. Plans = qué; Factory = cómo
5. Seis flujos; no menú de entidades
6. SQLite metadata + FS bytes; secrets solo OS store
7. MVP hash = **SHA-256**
8. Sin VigilCut en core; sin OmniRoute en núcleo

---

## 12. Escalabilidad (1000 tests / muchas pantallas)

- Pantallas viven bajo **6 flujos**, no 300 rutas primarias.
- Tests por módulo/flujo; no suite monstruo obligatoria en cada PR.
- Leer solo docs de capas tocadas.
- ADR para decisiones; no reescribir constituciones por feature.

---

## 13. Estado de implementación

| Hito | Estado |
|------|--------|
| **Foundation 0** | **Aprobada** (D-027) |
| **Foundation 1** | **Hecha** — SQLite WAL/FKs, migraciones, settings/paths (D-028) |
| Scaffold monorepo | Sí |
| Catálogo / Factory / Review | **Pausado** (D-026) |
| Jobs worker | Aún no |
