# Foundation Audit 01 — informe

**Fecha:** 2026-07-31
**Alcance:** calidad de fundación documental (sin producto nuevo, sin código de negocio).

---

## 1. Resumen ejecutivo

La fundación era **rica pero ruidosa**: dos series de constituciones, producto/arquitectura duplicados (PRODUCT vs 01, ARCHITECTURE vs 03), y restos de la decisión antigua de jobs (`generate → completed`).

**Audit 01** dejó:

- **8 autoridades** claras (playbook + product + architecture + 5 constituciones).
- Legado **explícitamente no normativo**.
- **D-019** alineado en workflows, database, jobs, plan.
- Metodología **menos burocrática** (multi-rol solo HIGH/ARCHITECTURE; DoR ligero en LOW).
- Reglas UX más **medibles** (preview ≥70%, selección con borde/fondo).

**No** se cambió MVP, stack, ni se implementó negocio.
**Foundation 0** quedó **aprobada** después de este audit (D-027).  
**Foundation 1** sigue pendiente de orden explícita.

---

## 2. Problemas encontrados

| # | Tipo | Problema |
|---|------|----------|
| P1 | Autoridad | Doble set de constituciones (F0 vs 00–09) sin deprecación formal |
| P2 | Autoridad | PRODUCT vs 01-PRODUCT; ARCHITECTURE vs 03-ARCHITECTURE |
| P3 | Contradicción | `05-DATABASE` aún decía generate job → completed |
| P4 | Contradicción | `04-WORKFLOWS` proponía job completed + asset waiting |
| P5 | Contradicción | Diagrama en `06-JOBS` solo success→completed |
| P6 | Contradicción | `11-IMPLEMENTATION_PLAN` Fase 15: “job completed + asset waiting” |
| P7 | Complejidad | Multi-rol obligatorio en **toda** tarea (no escala) |
| P8 | Complejidad | DoR de 8 ítems + leer 3 docs siempre (pesado para LOW) |
| P9 | Ambigüedad | “Demasiadas capas” sin umbral numérico |
| P10 | Ambigüedad | Selección “claramente visible”; preview “~70–75%” |
| P11 | QA | E2E Playwright webServer timeout (infra; no cerrado en este audit de docs) |
| P12 | Escalabilidad | Obligar lectura completa del histórico en cada tarea |

---

## 3. Problemas corregidos

| # | Corrección |
|---|------------|
| P1 | `constitution/README.md` + banners LEGACY en 00–09 |
| P2 | Headers de autoridad en PRODUCT/ARCHITECTURE; banners en 01/03 |
| P3–P6 | Textos alineados a **D-019** |
| P7 | Multi-rol solo HIGH/ARCHITECTURE en playbook + DONE |
| P8 | DoR ligero para LOW; lectura mínima por capas |
| P9 | STOP si ≥3 de {domain, persistence, jobs, backend, api, frontend} sin dividir |
| P10 | UX: ≥70% / ≤30%; selección con fondo o borde distinto |
| P12 | Playbook §12 escalabilidad; START-HERE “máximo 8 docs normativos” |
| — | Playbook reescrito más corto y con tabla de autoridad única |
| — | START-HERE y FOUNDATION_REVIEW actualizados |

---

## 4. Duplicaciones eliminadas / neutralizadas

| Duplicación | Acción |
|-------------|--------|
| Constituciones 00–09 vs F0 | Legado no normativo (no borrados: valor histórico) |
| 01-PRODUCT vs PRODUCT | 01 = referencia; PRODUCT = autoridad |
| 03-ARCHITECTURE vs ARCHITECTURE | 03 = referencia; ARCHITECTURE = autoridad |
| Invariantes repetidas en 10 sitios | Se mantienen en playbook (checklist corto) + ADR; no se reescribió todo el histórico |
| Multi-rol en playbook y DONE | Un solo criterio (HIGH+) |

**No se fusionaron** físicamente 01↔PRODUCT (pérdida de detalle); se fijó **una autoridad**.

---

## 5. Documentos modificados / creados

**Creados**

- `docs/constitution/README.md`
- `docs/FOUNDATION_AUDIT_01.md` (este informe)

**Reescritos / sustanciales**

- `docs/AI_PLAYBOOK.md`
- `docs/00-START-HERE.md`

**Actualizados**

- `docs/PRODUCT.md`, `docs/ARCHITECTURE.md`
- `docs/01-PRODUCT.md`, `docs/03-ARCHITECTURE.md`
- `docs/04-WORKFLOWS.md`, `docs/05-DATABASE.md`, `docs/06-JOBS.md`
- `docs/11-IMPLEMENTATION_PLAN.md`
- `docs/constitution/{ENGINEERING,UX_UI,TESTING,SECURITY,DONE}.md`
- `docs/constitution/00` … `09` (banner LEGACY)
- `docs/FOUNDATION_REVIEW.md`

**No modificados (producto/MVP/stack):** alcance MVP, decisiones D-019…D-026 de fondo, scaffold de código.

---

## 6. Riesgos pendientes

| Riesgo | Severidad | Notas |
|--------|-----------|--------|
| Alguien sigue editando 00–09 como verdad | Media | Banners + README; disciplina |
| E2E Playwright no verde | Media | Arreglar webServer en tarea infra |
| Histórico 01–13 aún puede confundir | Baja | START-HERE jerarquía |
| P-001/P-003/P-004 abiertas | Baja | Antes de Factory/Review finos |
| Encoding legacy en algunos 00–09 | Baja | No normativos |

---

## 7. Recomendaciones futuras

1. Aprobar Audit 01 + Foundation 0.
2. Tarea infra: Playwright `webServer` host/timeout.
3. Foundation 1: SQLite+WAL+FK+migraciones+settings (sin Factory).
4. No borrar 00–09 hasta que el equipo no los use; opcional archive folder más adelante.
5. CI cuando haya: `pnpm quality` + e2e + clippy `-D warnings`.
6. Cerrar P-001 antes de Manual Factory.

---

## 8. Validaciones

| Check | Resultado |
|-------|-----------|
| Cambio de MVP / arquitectura / stack | **No** |
| Código de negocio | **No** |
| Dependencias nuevas | **No** |
| Autoauditoría multi-rol | Ver §10 |
| Loop simplificación | Playbook acortado; multi-rol reducido |
| `git diff --check` | Ver entrega |

---

## 9. Autoauditoría multi-rol (post-fix)

| Rol | ¿Bloquearía? | Nota |
|-----|--------------|------|
| PM | No | MVP intacto; claridad de autoridad mejora delivery |
| PO | No | Aceptación y no-objetivos siguen en PRODUCT |
| Architect | No | Capas y D-019 consistentes |
| UX | No | Reglas más medibles |
| Backend | No | ENGINEERING autoridad clara |
| Frontend | No | Lectura mínima por tarea |
| QA | No* | *E2E gap sigue documentado, no es regresión de audit |
| Security | No | SECURITY intacta; D-023 vigente |

---

## 10. Metodología resultante (compacta)

```text
TASK CARD → DoR → (STOP?) → implement (orden capas si feature)
→ tests por riesgo → diff check → fix → simplify
→ multi-rol si HIGH+ → entrega sin commit → wait
```

Autoridades: **8 archivos** + ADR.
