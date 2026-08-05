# Providers — Visual Library

Catálogo y política de **generación de imágenes** (y chat de needs vía gateway).  
**No es un SDK.** La implementación vive en `crates/application/src/integrations/`.

| Doc | Contenido |
|-----|-----------|
| [00-PM-CTO-LOOPS.md](./00-PM-CTO-LOOPS.md) | Por qué este recorte del brief largo |
| [CATALOG.md](./CATALOG.md) | Tier-1 + matriz + fichas (estimated) |
| [BENCHMARK.md](./BENCHMARK.md) | 20 prompts oficiales VL |
| [SCORING.md](./SCORING.md) | Algoritmo de recomendación / pesos |
| [CONNECT-OMNIROUTE.md](./CONNECT-OMNIROUTE.md) | **Checklist e2e** gateway + smoke 3 prompts |

ADR: **D-039** en [12-DECISIONS.md](../12-DECISIONS.md).

---

## Taxonomía (`kind`)

| kind | Significado | Ejemplo |
|------|-------------|---------|
| `local_stub` | Sin red; tiles/determinista | `stub` |
| `gateway` | Enruta a otros backends | `omniroute` |
| `remote_api` | Vendor cloud HTTP | `openai-image`, `stability` |
| `local_runtime` | Proceso local (Comfy, A1111…) | *post-MVP* |

**Regla:** no puntuar un gateway como “calidad del modelo”; la calidad es del **backend** que enruta.

---

## Tiers

| Tier | Uso |
|------|-----|
| **0 Runtime** | Ya en código / Settings: `stub`, `omniroute`, `spacexai-image`, `openai-image`, `stability` |
| **1 Research** | Candidatos prioritarios documentados aquí (Fal, Replicate, BFL, HF, Cloudflare, …) |
| **2 Long-tail** | Local runtimes y agregadores; solo lista, sin ficha profunda |

---

## IDs runtime (contrato)

| `provider_id` | kind | Estado integración |
|---------------|------|--------------------|
| `stub` | local_stub | **Done** |
| `omniroute` | gateway | HTTP image + chat **listo para conectar** gateway |
| `spacexai-image` | remote_api | Key en Settings; HTTP **pendiente** |
| `openai-image` | remote_api | Key; HTTP **pendiente** |
| `stability` | remote_api | Key; HTTP **pendiente** |

Chat needs (no imagen): `script_ai_provider` = `heuristic` \| `omniroute` \| `spacexai`.

---

## Principios de selección (producto)

1. **Automatic:** free / barato primero (`omniroute_prefer_free`), luego calidad usable.  
2. **Manual:** preferred del need si runnable; si no, mismo ranking.  
3. **Siempre** hay fallback a `stub` en generación cuando el remoto falla (no perder el batch).  
4. **Presupuesto** por conector (¢ + free quota) manda sobre “el más bonito”.  
5. Nada entra a Library sin **Review**.

---

## Confianza de datos

| Tag | Significado |
|-----|------------|
| `measured` | Benchmark VL corrido en esta máquina/fecha |
| `estimated` | Docs públicos / reputación / pricing 2025–2026 aprox. |
| `unknown` | No afirmar |

Hasta correr [BENCHMARK.md](./BENCHMARK.md), **casi todo es `estimated`**.
