> **LEGACY — no normativo.** Fuente de verdad: [README.md](./README.md) y Foundation 0 (`docs/AI_PLAYBOOK.md`, `PRODUCT.md`, `ARCHITECTURE.md`, `ENGINEERING.md`, `UX_UI.md`, `TESTING.md`, `SECURITY.md`, `DONE.md`). Si hay conflicto, **ignorar este archivo**.

# 09 â€” Definition of Done

**Estado:** Normativo
Si un Ã­tem aplicable no se cumple, **no estÃ¡ Done**.

---

## 0. Informe de cierre

- Archivos modificados
- Resumen
- Pruebas ejecutadas / fallidas / omitidas (+ motivo)
- Riesgos
- Screenshots si UI real
- Commit/push solo con autorizaciÃ³n

---

## 1. Producto

- Objetivo aprobado Â· sin scope creep Â· estados/errores definidos Â· siguiente acciÃ³n clara Â· sin placeholders como feature

---

## 2. UX/UI

- ConstituciÃ³n UX Â· no overflow Â· empty/loading/error/success Â· selecciÃ³n visible Â· evidencia visual si aplica

---

## 3. Frontend

- Contratos tipados Â· sin any permanente Â· reconstruible Â· anti doble envÃ­o
- **Vitest** del cambio
- **Playwright** si toca flujo visible
- `pnpm build:ui`

---

## 4. Backend / Rust

- Invariantes Â· errores estructurados Â· transacciones Â· idempotencia Â· sin unwrap productivo
- **Obligatorio si tocÃ³ Rust:**
  - `cargo fmt`
  - `cargo check --workspace`
  - `cargo test --workspace`
- Clippy `-D warnings` cuando CI estable

---

## 5. Datos

- MigraciÃ³n **nueva** numerada (no editar publicadas)
- WAL + FKs
- SHA-256 (no pHash en MVP)
- Path safety

---

## 6. Jobs

- Durable Â· progress Â· cancel Â· retry Â· recovery
- **Generate â†’ `waiting_review`** (no `completed`)
- Outputs vÃ¡lidos Â· cleanup tmp

---

## 7. QA gates reales

```text
git diff --check
pnpm fmt:rust && pnpm check:rust && pnpm test:rust   # si Rust
pnpm test:ui && pnpm build:ui                        # si UI
pnpm test:e2e                                        # si flujo UI
```

---

## 8. Security / Observability / Docs

- Secrets solo OS store (proveedor real)
- Sin secrets en diff
- Docs/ADR actualizados
- Constituciones no alteradas para eludir

---

## 9. Bloqueo duro

1. Library sin Approve
2. Generate job `completed` en vez de `waiting_review`
3. Plan draft genera
4. VigilCut en core
5. Jobs solo memoria
6. Keys en SQLite/JSON/logs
7. Tests obligatorios fallidos u omitidos sin motivo

---

## 10. Referencias

- [00-ENGINEERING-CONSTITUTION.md](./00-ENGINEERING-CONSTITUTION.md) Â· D-019â€¦D-026
