> **LEGACY — no normativo.** Fuente de verdad: [README.md](./README.md) y Foundation 0 (`docs/AI_PLAYBOOK.md`, `PRODUCT.md`, `ARCHITECTURE.md`, `ENGINEERING.md`, `UX_UI.md`, `TESTING.md`, `SECURITY.md`, `DONE.md`). Si hay conflicto, **ignorar este archivo**.

# 06 â€” Testing Constitution

**Estado:** Normativo
**Principio:** Una funcionalidad sin pruebas no estÃ¡ terminada. Las pruebas cubren comportamiento.

---

## 1. Inventario de infraestructura (actualizado)

| Capa | Herramienta | Estado |
|------|-------------|--------|
| Unit/integration Rust | `cargo test` | Activo |
| Format Rust | `cargo fmt` | **Obligatorio** si toca Rust (D-024) |
| Check Rust | `cargo check` | **Obligatorio** si toca Rust (D-024) |
| Clippy | `cargo clippy -D warnings` | Gate cuando exista **CI estable** (D-024) |
| Typecheck TS | `pnpm build:ui` (`tsc --noEmit`) | Activo |
| Unit FE | **Vitest** + **Testing Library** | **Requerido / instalado** (D-020) |
| E2E | **Playwright** sobre **Vite** | **Requerido / instalado** (D-021) |
| E2E Tauri | â€” | Posterior, cuando haga falta |
| ESLint / Prettier | â€” | AÃºn no |
| CI | â€” | AÃºn no |

---

## 2. PirÃ¡mide

### Unit (Rust + FE)

- Dominio, transiciones, validadores, path rules, idempotency
- Componentes y helpers FE con mocks tipados
- Sin red, sin provider real, sin FS real, sin IA real

### Integration (Rust)

- SQLite temp, FS temp, jobs + persistencia, migraciones
- Cuando exista la capa de datos

### Smoke

- App/UI principal carga
- (Futuro) SQLite abre, migraciones, directorios

### E2E (Playwright + Vite)

- Flujos de UI sobre dev server Vite
- **No** requiere Tauri completo
- Sin providers reales

Flujo producto objetivo (cuando haya pantallas reales):

```text
seed/setup â†’ generate o import â†’ waiting_review â†’ approve
â†’ library search â†’ usage â†’ reload â†’ recuperar
```

E2E fallo: generate fail â†’ job failed â†’ retry.

### Regression

Bug corregido â†’ test que falla antes y pasa despuÃ©s.

---

## 3. Providers

| # | Ley |
|---|-----|
| T-P1 | Nunca providers reales en unit/integration/E2E normal. |
| T-P2 | Fake/stub only. |
| T-P3 | Pruebas reales de provider: separadas, env flag, lÃ­mite coste, nunca CI default. |

---

## 4. Comandos obligatorios por tipo de cambio

| Cambio | Comandos |
|--------|----------|
| Cualquiera | `git diff --check` |
| Rust | `cargo fmt` Â· `cargo check --workspace` Â· `cargo test --workspace` |
| UI | `pnpm --filter @visual-library/ui test` Â· `pnpm build:ui` |
| E2E UI (si toca flujos visibles) | `pnpm --filter @visual-library/ui test:e2e` |
| CI futuro | + `cargo clippy --workspace -- -D warnings` |

Scripts raÃ­z previstos:

```text
pnpm test              # FE unit + rust test
pnpm test:ui
pnpm test:e2e
pnpm test:rust
pnpm check:rust        # cargo check
pnpm fmt:rust          # cargo fmt
pnpm quality:rust      # fmt + check + test
```

---

## 5. Anti-patrones

- Declarar Done sin tests de la invariante
- Ejecutar suite irrelevante para â€œaparentarâ€
- Provider real en CI
- Omitir `cargo fmt` / `cargo check` en cambios Rust

---

## 6. Referencias

- `docs/10-QA.md` Â· `docs/12-DECISIONS.md` (D-020, D-021, D-024)
- [09-DEFINITION-OF-DONE.md](./09-DEFINITION-OF-DONE.md)
