# Catálogo Tier-1 (estimated)

**As-of:** 2026-08 · **confidence:** estimated (sin API calls en esta fase)  
**Uso:** elegir orden de integración y pesos; no sustituye ToS ni pricing en vivo.

Leyenda coste: **MB** muy barato · **B** barato · **M** medio · **C** caro · **MC** muy caro  
Riesgo: **MB** muy bajo … **MA** muy alto

---

## Matriz comparativa (resumen)

| id / nombre | kind | Coste | Calidad didáctica* | Velocidad* | API ease* | Free/tier | Comercial* | Batch* | VL fit |
|-------------|------|-------|--------------------|------------|-----------|-----------|------------|--------|--------|
| stub | local_stub | MB | baja (tile) | instant | n/a | sí | sí | sí | dev / offline |
| omniroute | gateway | MB–B | depende backend | varía | alta (OpenAI-compat) | vía backends | según backend | sí | **Automatic free** |
| openai-image | remote_api | C–MC | alta | media | alta | limitado | sí (ToS) | limitado | premium Manual |
| stability | remote_api | M | media–alta | media | media | a veces | sí | sí | genérica |
| spacexai-image | remote_api | M–C | alta (estim.) | media | media | no | revisar ToS | ? | calidad |
| fal.ai | remote_api | B–M | media–alta | **alta** | alta | créditos | sí | sí | velocidad |
| replicate | remote_api | M | media–alta | media | alta | free trial | sí | sí | muchos modelos |
| bfl / flux | remote_api | M–C | **alta** | media | media | no | sí | ? | calidad |
| google-imagen | remote_api | M–C | alta | media | media | GCP trial | sí | sí | realismo |
| together / fireworks | remote_api | B–M | media | alta | alta | créditos | sí | sí | open models |
| hf-inference | remote_api | MB–B | varía | varía | media | free lim. | varía | lim. | experiment |
| cloudflare-ai | remote_api | MB–B | media | alta | media | workers | sí | lim. | barato edge |
| comfyui local | local_runtime | MB† | alta si bien tunado | lenta setup | baja (ops) | hardware | sí | sí | power user |

\*estimated · †electricidad/GPU local  

---

## Runtime (Tier 0) — ficha corta

### stub
- **Sitio:** local VL  
- **API:** in-process  
- **Coste:** 0 · **Riesgo:** MB  
- **Caso VL:** tests, offline, demo de flujo Review  
- **Límite:** no sirve para Library “real”

### omniroute
- **Sitio:** [OmniRoute](https://github.com/diegosouzapw/OmniRoute) (gateway)  
- **API:** `/v1/images/generations`, `/v1/chat/completions`  
- **Tipo:** gateway local  
- **Coste:** MB–B (free stack si el backend lo es)  
- **Caso VL:** Automatic free-first; needs via Claude/chat  
- **Limitación:** calidad = backend enrutado; gateway down → fallback stub/heurística  
- **Riesgo:** Bajo (local) + riesgo del backend

### openai-image
- **Docs:** platform.openai.com images  
- **Coste:** C–MC · **Calidad didáctica:** alta  
- **Caso VL:** Manual premium, hero shots  
- **Riesgo:** Medio (precio, policy)

### stability
- **Docs:** platform.stability.ai  
- **Coste:** M · **Caso VL:** variedad open-ish models vía API  
- **Riesgo:** Medio

### spacexai-image
- **API:** xAI image (cuando conectada)  
- **Caso VL:** calidad alta si el adapter se completa  
- **Estado código:** key Settings; HTTP pendiente  
- **Riesgo:** Medio

---

## Tier 1 research (candidatos a `provider_id` futuro)

Ids **reservados** (no en Settings hasta adapter):

| future id | Vendor | Por qué VL | Prioridad integración |
|-----------|--------|------------|------------------------|
| `fal` | Fal.ai | Rápido, API clara, buen $ | P1 |
| `replicate` | Replicate | Zoo de modelos, easy HTTP | P1 |
| `bfl-flux` | Black Forest Labs | Calidad top | P2 |
| `google-imagen` | Google | Calidad; más fricción GCP | P3 |
| `together-image` | Together | Open models baratos | P2 |
| `fireworks-image` | Fireworks | Latencia | P2 |
| `hf-inference` | Hugging Face | Free/experiment | P3 |
| `cloudflare-ai` | Cloudflare | Barato edge | P3 |
| `comfyui` | ComfyUI local | Control total | P4 (ops) |

### Fal.ai (research)
- Cloud API, énfasis velocidad  
- **VL:** thumbnails y batch Automatic  
- **Coste:** B–M · **API:** buena · **Riesgo:** Bajo–Medio  

### Replicate
- Modelos versionados por slug  
- **VL:** probar Flux/SD sin ops  
- **Coste:** M · **Riesgo:** Medio (pricing por segundo)

### Black Forest Labs (Flux)
- Calidad alta perceived  
- **VL:** assets “premium” Manual  
- **Coste:** M–C · **Riesgo:** Medio  

### Google Imagen
- Alta calidad; setup GCP  
- **VL:** realismo si se justifica ops  
- **Riesgo:** Medio–Alto (cuenta/billing)

### Together / Fireworks
- Open weights hosted  
- **VL:** coste/latencia  
- **Riesgo:** Bajo–Medio  

### Hugging Face Inference
- Free tier limitado; modelos mixtos  
- **VL:** experimentación  
- **Riesgo:** Medio (cold start, limits)

### Cloudflare Workers AI
- Barato, edge  
- **VL:** gen simple barata  
- **Riesgo:** Bajo–Medio (calidad variable)

### ComfyUI / A1111 / Forge / Invoke (local_runtime)
- **No comparar ¢/img con cloud**  
- **VL:** usuarios avanzados, air-gap  
- **Riesgo ops:** Alto para MVP (instalación, GPU, paths)  
- **Decisión:** Tier 2 hasta que exista `local_runtime` port  

---

## Capacidades (MVP VL vs nice-to-have)

| Capacidad | MVP VL | Post |
|-----------|--------|------|
| Text-to-image | **sí** | |
| Seed | deseable | |
| Size / orientation | sí (metadata need) | |
| Negative prompt | | sí |
| Edit / inpaint | | sí |
| ControlNet / LoRA | | no prioritario |
| Multi-image ref | | sí |
| Transparent bg | | niche |

---

## Orden de integración recomendado (PO)

1. ~~stub~~ done  
2. **omniroute image** (validar free stack real)  
3. **openai-image** o **fal** (uno “calidad/velocidad” según presupuesto)  
4. **replicate** (flex modelos)  
5. Resto según benchmark  

Needs chat: **omniroute + Claude** (ya cableado a “conectar gateway”).
