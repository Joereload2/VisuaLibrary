# UX / UI Constitution

**Foundation 0** · Constitución visual e interactiva.
**Desktop first.** Verificable con checklist.

---

## 1. Principios

| # | Regla |
|---|--------|
| U1 | **Desktop first** (no mobile-first). Targets: 1366×768, 1440×900, 1920×1080. |
| U2 | Navegación por **6 flujos**: Factory, Review, Library, Coverage, Plans, Settings. |
| U3 | **Prohibido** menú primario de entidades (Conceptos, Assets, Representaciones). |
| U4 | Una **única acción primaria** visible por vista/contexto. |
| U5 | **Siguiente acción** siempre evidente (empty, error y success incluidos). |
| U6 | Selección **medible**: el ítem seleccionado tiene fondo o borde distinto del no seleccionado (no solo color de texto). |
| U7 | Lenguaje ubicuo de producto (Waiting Review, FOUND/GENERATE, Approve to Library). |

---

## 2. Layout y contenedor

| # | Regla |
|---|--------|
| L1 | Ningún componente crítico **fuera del contenedor principal** de la app. |
| L2 | **Sin overflow horizontal** en anchos desktop objetivo. |
| L3 | Si la tarea principal es **preview** de asset: el preview ocupa **≥70%** del área de trabajo útil (ideal 70–75%). |
| L4 | Paneles secundarios (metadata, cola, filtros): **≤30%** del área de trabajo útil. |
| L5 | Contenido principal tiene prioridad sobre chrome decorativo. |
| L6 | Responsive de **escritorio** (reflujo de paneles), no rediseño mobile. |

---

## 3. Estados obligatorios

Toda pantalla/feature de UI debe contemplar:

| Estado | Requisito |
|--------|-----------|
| Empty | Qué falta + CTA |
| Loading / running | Feedback inmediato; sin doble envío confuso |
| Success | Resultado claro (ej. “enviado a Waiting Review”) |
| Error | Mensaje humano + acción sugerida |
| Progress | Visible y coherente con snapshot (no “100% completed” engañoso) |
| Blocked | Path inválido, provider no configurado, plan no approved |

| # | Regla |
|---|--------|
| S1 | No asumir que un evento de progreso siempre llega; reconstruir desde estado real. |
| S2 | Tras reload, la vista se reconstruye desde datos persistidos. |
| S3 | Generate/job en revisión: UI dice **Waiting Review**, nunca “en Library”. |

---

## 4. Flujos (responsabilidades de UI)

| Estación | Sí | No |
|----------|----|----|
| Factory | Manual/Automatic, preview FOUND/GENERATE, progreso jobs | Aprobar a Library |
| Review | 5 acciones MVP, cola | Ser la Library |
| Library | Search/export approved | Generate / Approve |
| Coverage | Issues + CTA | Ejecutar generate |
| Plans | Qué crecer; approve plan | Llamar providers |
| Settings | Config | Producción |

---

## 5. Accesibilidad y consistencia

| # | Regla |
|---|--------|
| A1 | Focus visible. |
| A2 | No comunicar estado solo con color (FOUND vs GENERATE, etc.). |
| A3 | Confirmación en destructivas / masivas. |
| A4 | Preview con texto útil (concept + representation) cuando haya media. |
| A5 | Controles y espaciado consistentes entre estaciones. |
| A6 | Confirmación de bulk approve; nunca approve-all ciego por defecto. |

---

## 6. Evidencia visual (cuando la feature UI es real)

Capturas en **1366×768**, **1440×900**, **1920×1080**.

Validar en cada una: sin overflow horizontal · jerarquía · preview ratio · selección · acción primaria · estados relevantes.

Scaffold placeholder actual: no exige suite visual de negocio.

---

## 7. Checklist verificable (PR / Done UI)

```text
[ ] Desktop layout sin overflow horizontal (3 anchos si feature real)
[ ] Todo dentro del contenedor principal
[ ] Preview ≥70% si es tarea principal; secundarios ≤30%
[ ] Una sola acción primaria
[ ] Empty / loading / error / success definidos
[ ] Progress coherente (no completed engañoso en Waiting Review)
[ ] Selección con fondo/borde distinto (U6)

[ ] Siguiente acción evidente
[ ] Navegación solo por 6 flujos
[ ] Copy ubicuo de producto
[ ] a11y mínima (focus, no solo color, confirmaciones)
[ ] Responsive escritorio aceptable
[ ] Screenshots adjuntos si aplica DoD
```

---

## 8. Anti-patrones

- CRUD de Concepts como home
- Dashboard Coverage sin CTA
- Generate con toast “Guardado en Library”
- Panel de filtros más ancho que el preview
- Scroll horizontal por badges o tablas

---

## 9. Referencias

- `docs/PRODUCT.md` · `docs/AI_PLAYBOOK.md`
- Detalle histórico: `docs/09-UX.md`
