> **LEGACY — no normativo.** Fuente de verdad: [README.md](./README.md) y Foundation 0 (`docs/AI_PLAYBOOK.md`, `PRODUCT.md`, `ARCHITECTURE.md`, `ENGINEERING.md`, `UX_UI.md`, `TESTING.md`, `SECURITY.md`, `DONE.md`). Si hay conflicto, **ignorar este archivo**.

# 02 â€” Frontend Constitution

**Estado:** Normativo
**Ãmbito:** `packages/ui`
**Stack real:** React 19 Â· Vite 6 Â· TypeScript Â· React Router 7 Â· Vitest Â· Testing Library Â· Playwright (Vite)

---

## 1. Responsabilidad

### SÃ­

- presenta datos;
- permite decisiones;
- estado efÃ­mero de interacciÃ³n;
- consulta snapshots vÃ­a IPC;
- muestra progreso;
- retry y cancelaciÃ³n (cuando el backend lo exponga).

### No

- lÃ³gica de dominio;
- transiciones de jobs;
- SQL;
- control directo de providers;
- Ãºnica copia de datos importantes;
- dueÃ±a de la cola durable.

---

## 2. Estructura

```
packages/ui/src/
  app/
  flows/     # factory | review | library | coverage | plans | settings
  shared/
```

| # | Ley |
|---|-----|
| F-1 | Por flujos, no por tablas. |
| F-2 | Seis rutas primarias only. |
| F-3 | Infra de tests **antes** de pantallas de negocio (D-020). |

---

## 3. Estado

Separar: UI Â· datos persistidos (snapshot) Â· jobs Â· review Â· forms Â· selecciÃ³n/preview.
No store global monolÃ­tico. No segunda canonicidad frente a SQLite.

---

## 4. API / IPC

- Respuestas tipadas
- No `any`/`unknown` permanentes
- Commands = casos de uso
- Errores: `code`, `message`, `retryable`, `suggested_action`, `detail?`

---

## 5. Async

- Anti doble envÃ­o
- Feedback inmediato
- Refrescar snapshot
- Reconstruir tras reload
- Jobs solo por acciÃ³n explÃ­cita

---

## 6. Pruebas FE (D-020, D-021)

| Suite | Tool | CuÃ¡ndo |
|-------|------|--------|
| Unit/component | Vitest + Testing Library | Cada feature FE |
| E2E | Playwright + Vite | Flujos de UI |
| Tauri E2E | Posterior | Cuando se necesite |

Unit **no** usa red/provider/FS/IA reales.

```text
pnpm --filter @visual-library/ui test
pnpm --filter @visual-library/ui test:e2e
pnpm build:ui
```

---

## 7. Checklist PR

- [ ] Por flujo
- [ ] Tipos
- [ ] Tests Vitest del cambio
- [ ] E2E si toca flujo visible
- [ ] `pnpm build:ui`

---

## 8. Referencias

- `docs/07-FRONTEND.md` Â· [01-UX-UI-CONSTITUTION.md](./01-UX-UI-CONSTITUTION.md)
