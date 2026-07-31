> **LEGACY — no normativo.** Fuente de verdad: [README.md](./README.md) y Foundation 0 (`docs/AI_PLAYBOOK.md`, `PRODUCT.md`, `ARCHITECTURE.md`, `ENGINEERING.md`, `UX_UI.md`, `TESTING.md`, `SECURITY.md`, `DONE.md`). Si hay conflicto, **ignorar este archivo**.

# 00 â€” Engineering Constitution

**Estado:** Normativo (permanente)
**Stack real:** Tauri 2 Â· Rust workspace Â· React 19 Â· Vite 6 Â· TypeScript Â· pnpm 8 Â· Vitest Â· Playwright Â· SQLite/FS (diseÃ±ados)

Si la implementaciÃ³n contradice esta constituciÃ³n, la implementaciÃ³n estÃ¡ mal.

---

## 0. Protocolo antes de trabajar

1. Leer constituciones aplicables en `docs/constitution/`.
2. Identificar cuÃ¡les aplican.
3. Resumir reglas crÃ­ticas **antes** de tocar cÃ³digo.
4. Si contradice una constituciÃ³n â†’ detenerse y pedir aprobaciÃ³n.
5. **No** modificar constituciones para eludir reglas.
6. Tras implementar â†’ [09-DEFINITION-OF-DONE.md](./09-DEFINITION-OF-DONE.md).
7. No Done sin pruebas obligatorias.
8. Informar: ejecutadas Â· fallidas Â· omitidas Â· motivos.

---

## 1. Objetivo del sistema

Local-first Â· operable por una persona Â· recuperable Â· simple de mantener Â· extensible Â· segura con datos/archivos Â· automatizada pero supervisable.

---

## 2. JerarquÃ­a de verdad

1. Constituciones
2. Non-goals
3. Decisiones (`docs/12-DECISIONS.md`)
4. Producto / dominio / workflows
5. Arquitectura y plan
6. CÃ³digo

---

## 3. Identidad de producto

| # | Ley |
|---|-----|
| E-P1 | Independiente de VigilCut. |
| E-P2 | Concept â†’ Representation â†’ Asset â†’ Usage. |
| E-P3 | Seis flujos: Factory, Review, Library, Coverage, Plans, Settings. |
| E-P4 | Plans = quÃ©; Factory = cÃ³mo. |
| E-P5 | Generate â†’ waiting_review (job + asset); solo Approve â†’ Library. |
| E-P6 | Sin OmniRoute/IA de negocio sin fase aprobada. Stub primero. |
| E-P7 | Producto en **pausa de features** hasta nueva orden (D-026); sÃ­ infra y docs. |

---

## 4. Reglas generales (E-1 â€¦ E-15)

1. No feature sin objetivo, flujo, estados, aceptaciÃ³n, pruebas.
2. No abstracciones especulativas.
3. No microservicios/cloud/daemon sin aprobaciÃ³n.
4. No duplicar fuente de verdad.
5. No canonicidad paralela memoria/FE/JSON/SQLite.
6. UI presenta y gobierna; no es dueÃ±a de jobs/datos.
7. SQLite = metadata/estados.
8. FS = bytes.
9. Trabajo largo durable/cancelable/recuperable.
10. Archivo final validado antes de completado.
11. Cambios pequeÃ±os, probados, reversibles.
12. No mezclar gran refactor + feature.
13. No borrar legacy sin reemplazo probado y aprobado.
14. No Done con tests fallidos, estados a medias, errores ocultos, placeholders como feature.
15. AmbigÃ¼edad de producto/datos/arquitectura â†’ pedir aprobaciÃ³n.

---

## 5. Decisiones de infra aprobadas (resumen)

| ID | DecisiÃ³n |
|----|----------|
| D-019 | Generate jobs â†’ **`waiting_review`**, no `completed` |
| D-020 | Vitest + Testing Library desde el inicio |
| D-021 | Playwright sobre Vite (Tauri E2E despuÃ©s) |
| D-022 | MVP: solo SHA-256; pHash post-MVP |
| D-023 | Secrets en OS secure store; nunca SQLite/JSON/plano/logs |
| D-024 | `cargo fmt` + `cargo check` obligatorios; clippy en CI |
| D-025 | WAL + FKs + migraciones numeradas inmutables publicadas |
| D-026 | Negocio en pausa; solo infra/docs |

---

## 6. Herramientas y comandos reales

```text
# Siempre
git diff --check

# Rust (si toca crates / tauri)
pnpm fmt:rust          # cargo fmt --all
pnpm check:rust        # cargo check --workspace
pnpm test:rust         # cargo test --workspace
pnpm quality:rust      # fmt + check + test

# Frontend
pnpm test:ui           # vitest
pnpm test:e2e          # playwright (vite)
pnpm build:ui

# Desktop dev (manual)
pnpm dev
```

Clippy `-D warnings`: gate cuando CI estable exista.

---

## 7. Anti-patrones

- Generate job `completed`
- Placeholder como feature Done
- API keys en SQLite/JSON
- pHash en MVP
- Editar migraciÃ³n publicada
- Omitir fmt/check en cambios Rust

---

## 8. Constituciones hijas

01 UX Â· 02 FE Â· 03 BE Â· 04 Data Â· 05 Jobs Â· 06 Testing Â· 07 Security Â· 08 Observability Â· 09 DoD

---

## 9. Referencias

- `docs/00-START-HERE.md` Â· `docs/12-DECISIONS.md` Â· `docs/13-NON_GOALS.md`
