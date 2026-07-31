> **LEGACY — no normativo.** Fuente de verdad: [README.md](./README.md) y Foundation 0 (`docs/AI_PLAYBOOK.md`, `PRODUCT.md`, `ARCHITECTURE.md`, `ENGINEERING.md`, `UX_UI.md`, `TESTING.md`, `SECURITY.md`, `DONE.md`). Si hay conflicto, **ignorar este archivo**.

# 01 Ã¢â‚¬â€ UX / UI Constitution

**Estado:** Normativo
**ÃƒÂmbito:** NavegaciÃƒÂ³n, estaciones, estados de interfaz, copy, evidencia visual
**Stack UI real:** React 19 + React Router 7 en `packages/ui` (shell Tauri)

Complementa `docs/09-UX.md` y `docs/04-WORKFLOWS.md`.

---

## 1. Objetivo UX

El usuario debe poder **operar solo** el ciclo:

```
Settings Ã¢â€ â€™ (Plans | Manual Factory) Ã¢â€ â€™ Factory Ã¢â€ â€™ Review Ã¢â€ â€™ Library Ã¢â€ â€™ Coverage Ã¢â€ â€™ Plans Ã¢â‚¬Â¦
```

sin administrar tablas del dominio como menÃƒÂº principal.

---

## 2. NavegaciÃƒÂ³n

| # | Ley |
|---|-----|
| U-1 | NavegaciÃƒÂ³n primaria = **exactamente 6** estaciones: Factory, Review, Library, Coverage, Plans, Settings. |
| U-2 | **Prohibido** menÃƒÂº primario: Conceptos, Representaciones, Assets, Requests, Jobs (Jobs puede ser widget, no 7Ã‚Âº flujo). |
| U-3 | Sub-navegaciÃƒÂ³n solo **dentro** del flujo (`/factory/manual`, `/factory/automatic`, detalle de plan, etc.). |
| U-4 | Deep links son CTA de flujo (Coverage Ã¢â€ â€™ Plans, Factory Ã¢â€ â€™ Review), no admin oculto. |
| U-5 | Badges (cola Review, jobs failed) no crean flujos nuevos. |
| U-6 | El contenido principal tiene prioridad visual sobre chrome y decoraciÃƒÂ³n. |
| U-7 | **No overflow horizontal** en anchos de escritorio objetivo (ver Ã‚Â§7). |

Rutas reales del scaffold:

```
/factory/manual | /factory/automatic
/review
/library
/coverage
/plans
/settings
```

---

## 3. Responsabilidad de la UI

La UI **sÃƒÂ­**:

- presenta datos y snapshots persistidos;
- permite decisiones (approve, cancel, retry, editar forms);
- mantiene estado **efÃƒÂ­mero** de interacciÃƒÂ³n;
- muestra progreso;
- gobierna el flujo (quÃƒÂ© pantalla, quÃƒÂ© confirmaciÃƒÂ³n).

La UI **no**:

- es dueÃƒÂ±a de jobs o del catÃƒÂ¡logo;
- inventa estados de dominio no persistidos;
- presenta placeholders como funcionalidad terminada (E-14).

---

## 4. Estados visibles obligatorios por feature de estaciÃƒÂ³n

Toda feature de UI debe diseÃƒÂ±ar y mostrar, cuando aplique:

| Estado | Requisito |
|--------|-----------|
| **empty** | QuÃƒÂ© falta y siguiente acciÃƒÂ³n |
| **loading / running** | Feedback inmediato; sin doble envÃƒÂ­o confuso |
| **success** | Resultado claro (p.ej. Ã¢â‚¬Å“N en Waiting ReviewÃ¢â‚¬Â) |
| **error** | Mensaje humano + acciÃƒÂ³n sugerida |
| **interrupted** | Trabajo recuperable / reintentable cuando aplique |
| **retry / cancel** | Controles cuando el backend lo soporte |
| **blocked** | Permisos, path invÃƒÂ¡lido, provider no configurado |
| **selection visible** | SelecciÃƒÂ³n de filas/assets inequÃƒÂ­voca |

| # | Ley |
|---|-----|
| U-10 | No asumir que un evento de progreso siempre llegarÃƒÂ¡; poder reconstruir desde snapshot. |
| U-11 | Tras recargar la app, la pantalla se reconstruye desde estado persistido (no solo memoria). |
| U-12 | No guardar Ã¢â‚¬Å“segunda etapa del flujoÃ¢â‚¬Â en UI si se puede derivar del estado real en SQLite. |

---

## 5. Leyes por estaciÃƒÂ³n

### Factory

| # | Ley |
|---|-----|
| U-20 | Manual vs Automatic claramente separados. |
| U-21 | Manual = lista estructurada de necesidades (no prompt playground libre). |
| U-22 | Preview FOUND/GENERATE antes de gastar provider cuando el flujo lo permita. |
| U-23 | Automatic solo planes **approved**. |
| U-24 | Tras generate: copy a **Waiting Review**, nunca Ã¢â‚¬Å“ya en LibraryÃ¢â‚¬Â. |
| U-25 | FOUND muestra el asset reutilizado. |
| U-26 | Jobs se inician por **acciÃƒÂ³n explÃƒÂ­cita** del usuario. |

### Review

| # | Ley |
|---|-----|
| U-30 | Cola de decisiÃƒÂ³n, no Library. |
| U-31 | Acciones MVP: Approve, Reject, Edit metadata, Regenerate, Mark duplicate. |
| U-32 | Bulk approve solo con selecciÃƒÂ³n + confirmaciÃƒÂ³n (nunca approve-all ciego por defecto). |
| U-33 | Tras Approve: feedback Ã¢â‚¬Å“en LibraryÃ¢â‚¬Â y sale de cola. |

### Library

| # | Ley |
|---|-----|
| U-40 | Solo **approved**. |
| U-41 | Sin Generate / Approve / Reject. |
| U-42 | Buscar, filtrar, consultar, exportar informaciÃƒÂ³n. |

### Coverage

| # | Ley |
|---|-----|
| U-50 | Issues accionables (no solo charts). |
| U-51 | Sin ejecutar generaciÃƒÂ³n desde Coverage. |

### Plans

| # | Ley |
|---|-----|
| U-60 | Define **quÃƒÂ©**; no genera. |
| U-61 | Approve habilita Automatic Factory; CTA de ejecuciÃƒÂ³n va a Factory Automatic. |
| U-62 | Sin llamar providers. |

### Settings

| # | Ley |
|---|-----|
| U-70 | Solo configuraciÃƒÂ³n. Sin producciÃƒÂ³n. |
| U-71 | Paths y providers con validaciÃƒÂ³n visible y errores accionables. |

---

## 6. Lenguaje (microcopy)

| Preferir | Evitar |
|----------|--------|
| Waiting Review | Pending ambiguo |
| FOUND / GENERATE | Skip/Create confuso |
| Approve to Library | Publish online |
| Plan approved | Plan running |
| Coverage issue | Insight vacÃƒÂ­o |

Usar lenguaje ubicuo de `docs/02-DOMAIN.md`.

---

## 7. Evidencia visual (cuando la feature tenga UI real)

Para pantallas principales de una feature **Done**, entregar capturas en:

- 1366Ãƒâ€”768
- 1440Ãƒâ€”900
- 1920Ãƒâ€”1080

Validar: sin overflow horizontal Ã‚Â· jerarquÃƒÂ­a Ã‚Â· preview Ã‚Â· selecciÃƒÂ³n Ã‚Â· acciÃƒÂ³n primaria Ã‚Â· estados Ã‚Â· responsive de **escritorio** (no mobile-first).

**Hoy (Fase 1):** el shell es placeholder por estaciÃƒÂ³n; no se exige suite visual hasta features reales de cada flujo.

---

## 8. Accesibilidad mÃƒÂ­nima

| # | Ley |
|---|-----|
| U-80 | Focus visible. |
| U-81 | No distinguir FOUND/GENERATE solo por color. |
| U-82 | ConfirmaciÃƒÂ³n en acciones destructivas o masivas. |
| U-83 | Preview con texto alternativo (concept + representation) cuando haya media. |

---

## 9. Checklist PR UX

- [ ] 6 flujos only
- [ ] Estados empty/loading/error/success definidos
- [ ] Sin overflow horizontal en desktop targets
- [ ] SelecciÃƒÂ³n y acciÃƒÂ³n primaria claras
- [ ] Copy ubicuo
- [ ] No placeholder vendido como done
- [ ] Screenshots si la DoD de la tarea lo exige

---

## 10. Referencias

- `docs/09-UX.md` Ã‚Â· `docs/04-WORKFLOWS.md` Ã‚Â· `docs/01-PRODUCT.md`
- [02-FRONTEND-CONSTITUTION.md](./02-FRONTEND-CONSTITUTION.md)
