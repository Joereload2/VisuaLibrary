# TESTING Constitution

**Foundation 0** · Estrategia oficial de pruebas.
Solo herramientas **reales** del repo.

---

## 1. Principio

Una funcionalidad sin las pruebas de su **nivel de riesgo** no está terminada.
Probar **comportamiento**, no “líneas por rellenar”.
**No** ejecutar suites irrelevantes. **No** omitir las obligatorias.

---

## 2. Inventario real

| Tipo | Herramienta | Comando |
|------|-------------|---------|
| Unit/integration Rust | `cargo test` | `pnpm test:rust` |
| Format Rust | `cargo fmt` | `pnpm fmt:rust` / `fmt:rust:check` |
| Check Rust | `cargo check` | `pnpm check:rust` |
| Unit FE | **Vitest** + Testing Library | `pnpm test:ui` |
| E2E UI | **Playwright** + **Vite** | `pnpm test:e2e` |
| Typecheck/build UI | `tsc` + Vite | `pnpm build:ui` |
| Diff hygiene | git | `git diff --check` |
| Clippy | — | Gate **cuando CI estable** (`-D warnings`) |
| ESLint / Prettier / GitHub Actions | **No** | Hueco documentado |

Providers reales / IA: **prohibidos** en unit, integration y E2E normal. Usar stubs.

---

## 3. Tipos de prueba

| Tipo | Qué valida | Dónde |
|------|------------|-------|
| **Unit** | Reglas puras, transiciones, parsers, componentes aislados | `cargo test` domain; Vitest |
| **Integration** | SQLite temp, FS temp, jobs+DB, use case+repos | `cargo test` infrastructure/application (cuando existan) |
| **Smoke** | Arranca lo esencial sin estar roto | build/check; shell E2E; health |
| **E2E** | Flujo de usuario en UI (Vite) | Playwright |
| **Regression** | Bug que no vuelve | test que fallaba antes del fix |

E2E Tauri completo: **posterior** (no bloquea hoy).

---

## 4. Pruebas según riesgo

| Riesgo | Obligatorio |
|--------|-------------|
| **LOW** | Si hay código: unit del cambio. Siempre `git diff --check`. Docs-only: solo diff check. |
| **MEDIUM** | Unit · Integration **solo si** hay I/O/DB/FS · `git diff --check` · fmt/check si Rust · build:ui si TS |
| **HIGH** | MEDIUM + E2E del **flujo tocado** (si UI) · regression si bugfix |
| **ARCHITECTURE** | HIGH aplicable + ADR/docs + `pnpm quality` (o equivalente por capas) |

### Matriz práctica (comandos)

| Riesgo | Comandos típicos |
|--------|------------------|
| LOW (solo docs) | `git diff --check` |
| LOW (helper TS) | `pnpm test:ui` · `git diff --check` |
| MEDIUM (Rust use case) | `pnpm fmt:rust` · `pnpm check:rust` · `pnpm test:rust` · `git diff --check` |
| MEDIUM (UI flujo) | `pnpm test:ui` · `pnpm build:ui` · `git diff --check` |
| HIGH (Factory/Review UI+API) | quality Rust + `pnpm test:ui` + `pnpm test:e2e` + `pnpm build:ui` + `git diff --check` |
| ARCHITECTURE | `pnpm quality` + E2E + docs |

Si una herramienta **no aplica** (ej. no hay UI): marcar **N/A** en la entrega, no fingir.

---

## 5. Estados de comportamiento a cubrir (cuando aplique)

happy path · input inválido · empty · loading/running · cancel · fail · retry · interrupt/restart · duplicación · paths/permisos inválidos.

Generate: asertar job/asset en **`waiting_review`**, no Library.

---

## 6. Migraciones (cuando existan)

Empty DB · upgrade desde publicada · re-run seguro · FKs · no editar migración publicada · preservación de datos.

---

## 7. E2E canónico (cuando haya negocio)

```text
setup → generate/import → waiting_review → approve
→ library search → usage → reload → recuperar
```

Fallo:

```text
generate fail → failed → retry → recuperar
```

Hoy: smoke de **shell** (6 estaciones placeholder) en `packages/ui/e2e`.

---

## 8. Reglas de ejecución

| # | Regla |
|---|--------|
| T1 | Ejecutar solo lo que puede detectar regresión del **diff**. |
| T2 | No llamar providers reales / OmniRoute en CI o suite normal. |
| T3 | Temp dirs para SQLite/FS. |
| T4 | Bug fix importante → regression test. |
| T5 | Informar en entrega: ejecutadas / fallidas / omitidas + motivo. |
| T6 | Si tocó Rust: fmt + check + test son mandatorios (riesgo ≥ que toque código Rust). |

---

## 9. Anti-patrones

- “Lo probé a mano” como único gate de invariante
- Suite completa irrelevante para un cambio de un archivo de docs
- E2E con API de pago
- Omitir E2E en HIGH “por tiempo” sin aprobación

---

## 10. Referencias

- `docs/AI_PLAYBOOK.md` · `DONE.md`
- `docs/10-QA.md` (detalle histórico)
