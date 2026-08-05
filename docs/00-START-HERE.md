# START HERE — Visual Library

## Leer solo esto para empezar

| # | Documento | Autoridad de |
|---|-----------|----------------|
| 1 | [AI_PLAYBOOK.md](./AI_PLAYBOOK.md) | **Metodología** (cómo trabajar) |
| 2 | [PRODUCT.md](./PRODUCT.md) | **Producto** (MVP, flujos, no-objetivos) |
| 3 | [ARCHITECTURE.md](./ARCHITECTURE.md) | **Arquitectura** |
| 4 | [constitution/](./constitution/README.md) | **Ingeniería, UX, Testing, Security, Done** |

**Máximo 8 archivos normativos.** No hace falta leer el resto para trabajar.

---

## Jerarquía de verdad (si hay conflicto)

1. `docs/12-DECISIONS.md` (ADR aceptados)
2. Foundation 0: AI_PLAYBOOK, PRODUCT, ARCHITECTURE, constitution/{ENGINEERING,UX_UI,TESTING,SECURITY,DONE}
3. Referencia profunda: `docs/02`–`11`, `13`
4. Legado: `docs/01`, `constitution/00`–`09` → **no normativos**
5. Código

Mapa histórico: [FOUNDATION_REVIEW.md](./FOUNDATION_REVIEW.md).

---

## Estado del repo

| | |
|--|--|
| **Foundation 0** | **Aprobada** |
| **Foundation 1** | **Hecha** — SQLite + WAL/FKs + migraciones + settings/paths |
| **Foundation 2** | **Hecha** — domain gates + tablas catálogo + ensure/list concepts |
| **Foundation 3** | **Hecha** — generate stub → waiting_review + Review approve/reject + Library approved |
| **Foundation 4** | **Hecha** — Manual Factory preview FOUND/GENERATE + submit faltantes |
| **Foundation 5** | **Hecha** — Plans draft/approve + Automatic Factory |
| **Foundation 6** | **Hecha** — Coverage issues + Review 5 acciones |
| Scaffold Tauri + 6 rutas | Sí (estaciones con UI real F1–F6) |
| Negocio (Factory/Review/…) | **MVP local usable** (stub + OmniRoute e2e; catálogo providers en docs) |
| Tests | Vitest, Playwright (Vite), cargo test/fmt/check |
| Quality gate | `pnpm quality` · [reglas-calidad-codigo.md](./reglas-calidad-codigo.md) · **D-040** |
| **Handoff guiones** | [SCRIPT-FEEDER-HANDOFF.md](./SCRIPT-FEEDER-HANDOFF.md) — contrato para la app hermana que alimenta guiones |

### Congelación de foco (handoff)

VL queda **lista para consumir guiones** vía Manual Factory (paste de texto).  
La **siguiente app** del roadmap de producto es el **alimentador de guiones** (idea → texto de lección).  
Al retomar VL: providers de volumen (Fal/etc.), import de archivo, y bridge según el handoff.

---

## Referencia profunda (solo bajo demanda)

| Necesitas | Abre |
|-----------|------|
| Entidades detalladas | `02-DOMAIN.md` |
| Workflows paso a paso | `04-WORKFLOWS.md` |
| Esquema SQL propuesto | `05-DATABASE.md` |
| Jobs detallados | `06-JOBS.md` |
| Plan por fases | `11-IMPLEMENTATION_PLAN.md` |
| Non-goals largos | `13-NON_GOALS.md` |
| Providers imagen (catálogo, benchmark, scoring) | [`providers/README.md`](./providers/README.md) · **D-039** |
| Calidad de código (estilo + `pnpm quality`) | [`reglas-calidad-codigo.md`](./reglas-calidad-codigo.md) · **D-040** |
| App de guiones → VL | [`SCRIPT-FEEDER-HANDOFF.md`](./SCRIPT-FEEDER-HANDOFF.md) |
