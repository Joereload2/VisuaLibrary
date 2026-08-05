# Benchmark oficial Visual Library

**Objetivo:** comparar providers con **los mismos 20 prompts**, orientados a lección YouTube / conceptos educativos.  
**Estado:** definición lista; **runs medidos = pendiente** (requiere adapters + approve).

Idioma de prompt: **inglés** (mejor cobertura multi-modelo) + nota pedagógica en ES en el id.

---

## Protocolo

| Parámetro | Valor |
|-----------|--------|
| Seeds | Fijos: `vl-bench-01` … `vl-bench-20` (o seed numérico derivado del id) |
| Size | Preferir 1024×1024 o el default del provider más cercano |
| Runs por prompt | 1 (smoke) o 3 (media) |
| Negative | Ninguno salvo que el API lo exija |
| Timeout | 90s |
| Fallback stub | **No** contar como éxito del provider bajo test |

### Métricas por imagen

| Métrica | Tipo | Cómo |
|---------|------|------|
| latency_ms | objetiva | wall clock |
| cost_cents | objetiva | ledger / tarifa documentada |
| http_ok | objetiva | 2xx + bytes > umbral |
| resolution | objetiva | w×h |
| prompt_fidelity | 1–5 humana | ¿se entiende el concepto? |
| didactic_clarity | 1–5 humana | ¿sirve en lección? |
| artifact_hands | 0–2 | 0 ok / 1 dudoso / 2 roto (si aplica) |
| text_in_image | 0–2 | overlays ilegibles |
| review_pass | bool | ¿aprobarías a Library? |

### Agregados por provider

- media latency, p95  
- coste total 20 prompts  
- % http_ok  
- media fidelity / didactic  
- % review_pass  
- coste anual estimado = (coste_20 / 20) × generaciones/mes × 12 (hipótesis de uso en doc de run)

---

## 20 prompts estándar

| # | id | Tema | Prompt (EN) |
|---|-----|------|-------------|
| 1 | `econ-supply-demand` | economía | Simple educational diagram of supply and demand curves meeting at equilibrium, clean white background, didactic illustration, no text labels |
| 2 | `money-coins` | dinero | Clear didactic illustration of coins and a piggy bank, single subject, soft light, no watermarks, educational style |
| 3 | `office-desk` | oficina | Modern clean office desk with laptop and notebook, educational stock style, bright, uncluttered |
| 4 | `business-handshake` | negocios | Professional handshake silhouette concept, abstract corporate education illustration, minimal |
| 5 | `family-learning` | familia | Parents and child reading a book together, warm didactic illustration, respectful, no logos |
| 6 | `medicine-heart` | medicina | Anatomical heart educational illustration, clear sections, textbook style, no gore |
| 7 | `nature-tree` | naturaleza | Cross-section of a tree showing roots trunk leaves, educational diagram style, soft colors |
| 8 | `food-plate` | comida | Balanced meal plate illustration for nutrition lesson, top view, clean, appetizing didactic |
| 9 | `city-map` | ciudad | Simplified isometric city block for urban planning lesson, clear shapes, no tiny unreadable text |
| 10 | `tech-chip` | tecnología | Stylized CPU chip macro illustration for tech literacy, clean edges, blue-gray palette |
| 11 | `portrait-teacher` | retratos | Friendly adult teacher portrait illustration, diverse, school context, not photoreal celebrity |
| 12 | `object-microscope` | objetos | School microscope on white background, product-educational style, sharp silhouette |
| 13 | `vehicle-bus` | vehículos | Public school bus side view illustration, simple, bright, didactic |
| 14 | `architecture-bridge` | arquitectura | Suspension bridge structural concept illustration, clear cables and towers, educational |
| 15 | `pet-dog` | mascotas | Friendly dog sitting, educational children's encyclopedia style, no breed text |
| 16 | `sport-football` | deportes | Soccer ball and goal concept, clean sports education illustration |
| 17 | `education-classroom` | educación | Empty bright classroom with chalkboard and desks, wide shot, inviting, no readable writing on board |
| 18 | `travel-passport` | viajes | Passport and suitcase minimal travel concept, flat didactic illustration |
| 19 | `finance-chart` | finanzas | Abstract upward bar chart and coin, personal finance education, simple shapes |
| 20 | `industry-factory` | industria | Simplified factory with conveyor and gears, safety-positive industrial education illustration |

---

## Plantilla de run (rellenar al ejecutar)

```markdown
## Run YYYY-MM-DD
- provider_id:
- model:
- machine:
- notes:

| id | latency_ms | cost_cents | http_ok | fidelity | didactic | review_pass |
|----|------------|------------|---------|----------|----------|-------------|
| econ-supply-demand | | | | | | |
...
```

Guardar runs bajo `docs/providers/runs/` cuando existan (git opcional / local).
