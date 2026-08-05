# Scoring — recomendación de provider

Implementación runtime: `score_image_provider` + `select_image_provider_with_config`  
en `crates/application/src/integrations/image_gen.rs`.

Los pesos son **configurables en código** (constantes); la UI de pesos es post-MVP.

---

## Dimensiones (0–100 internas)

| Dimensión | Fuente hoy | Notas |
|-----------|------------|--------|
| **free_bonus** | ledger `is_free` / unit cost 0 | Automatic lo prioriza |
| **quality** | `quality_score` catálogo | Estimated hasta benchmark |
| **cost** | invertido de `cost_score` (0=barato) | Menor cost_score → mayor score |
| **availability** | `availability_score` + status ready | Gateway up, keys, budget |
| **runnable** | enabled ∧ can_afford ∧ status | Gate duro: score 0 si no |

No usamos aún (reservado post-benchmark): prompt_fidelity, hands, text_in_image, error_rate.

---

## Perfiles de peso

### Automatic (default producto)

| Peso | Valor |
|------|------:|
| free | 40 |
| cost | 25 |
| availability | 20 |
| quality | 15 |

### Manual (cuando hay preferred usable)

Preferred **gana** si es runnable (no se re-scorea en contra del usuario).  
Si preferred cae: perfil Automatic.

### Premium (futuro)

quality 45 · free 10 · cost 15 · availability 30 — no activo.

---

## Algoritmo (pseudocódigo)

```
if !runnable(p): score = -∞
else:
  cost_component = (100 - cost_score)           # cost_score 0..100, bajo = barato
  free_component = is_free ? 100 : 0
  score = w.free * free_component
        + w.quality * quality_score
        + w.cost * cost_component
        + w.availability * availability_score
  score /= (w.free + w.quality + w.cost + w.availability)
```

Desempate: `id` lexicográfico estable.

---

## Especialización (futuro)

Tabla opcional por `use_case` (thumbnail, didactic, portrait, diagram):

```
final = 0.85 * score + 0.15 * specialization[use_case]
```

No implementado hasta tener datos de benchmark.

---

## Relación con presupuestos

Si `!can_afford_one` → no runnable (score no aplica).  
Reset de uso y límites: Settings → Presupuesto.
