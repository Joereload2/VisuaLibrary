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
| Negocio (Factory/Review/…) | **MVP local usable** (stub de generación; sin providers IA) |
| Tests | Vitest, Playwright (Vite), cargo test/fmt/check |

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
