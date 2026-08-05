# Provider Catalog — loops PM / PO / CTO

**Fecha:** 2026-08-02  
**Entrada:** brief “PROVIDER CATALOG FOUNDATION” (sugerencia de investigación exhaustiva).  
**Salida:** qué aplicamos a Visual Library y qué no.

---

## Loop 1 — Diagnóstico

### PM (producto)

| Pregunta | Respuesta VL |
|----------|----------------|
| ¿Qué problema resuelve el catálogo? | Elegir **cómo** generar sin romper Factory→Review→Library ni presupuestos. |
| ¿Usuario? | Creador local (YouTube/educación), no ops de multi-cloud genérico. |
| ¿Éxito? | Automatic barato/free; Manual con calidad suficiente; Review sigue siendo gate. |
| ¿Overkill del brief? | Catálogo “infinito” + scores de manos/texto **sin medir** = ruido. |

**Aplicable:** matriz Tier-1, costes/bandas, casos de uso VL, scoring con pesos, benchmark de prompts fijos.  
**No aplicable ahora:** implementar providers, keys, APIs, ControlNet/LoRA en MVP, paper de 40 vendors.

### PO (prioridad)

1. Documentar **gateway vs generator** (OmniRoute ≠ modelo de imagen).  
2. Anclar IDs al código (`stub`, `omniroute`, `openai-image`, …).  
3. Benchmark **educación / YouTube** (20 prompts), no genérico stock.  
4. Scoring **Automatic = free/cost first**; Manual = quality/fidelity.  
5. Dejar **Provider SDK** como fase posterior con aprobación.

### CTO (arquitectura)

| Riesgo del brief | Mitigación VL |
|------------------|---------------|
| Dual world docs ≠ código | Catálogo runtime pequeño; research en `docs/providers/`. |
| Local vs cloud incomparables | Taxonomía `kind`: `local_stub` \| `gateway` \| `remote_api` \| `local_runtime`. |
| Calidad inventada | Etiqueta `confidence: estimated` hasta benchmark real. |
| Scope creep SDK | Sin HTTP nuevo; solo scoring + docs + ADR. |

---

## Loop 2 — Decisiones de implementación

### Hacer ahora

| Entrega | Por qué |
|---------|---------|
| `docs/providers/*` | Fuente de verdad de investigación Tier-1 + benchmark + scoring. |
| **D-039** | ADR: catálogo foundation, tiers, no SDK aún. |
| Scoring en `select_image_provider_with_config` | Alinea runtime con pesos documentados (mejora real). |
| Enlace desde START-HERE | Descubribilidad. |

### No hacer ahora

- Implementar Fal/Replicate/Comfy/etc.  
- Llamar APIs ni guardar keys de research.  
- Matriz de 50 providers con “calidad manos” como si estuviera medida.  
- Cambiar UX de Settings a un lab de providers.

### Criterios de “Tier-1 para VL”

Entrar en Tier-1 solo si cumple **≥4**:

1. API documentada o gateway OpenAI-compatible  
2. Uso comercial razonable para YouTube  
3. Precio o free-tier entendible  
4. Útil para **ilustración didáctica** (no solo foto fashion)  
5. Encaja local-first (key en máquina / proceso local)  
6. Mapeable a un `provider_id` estable  

---

## Roles: veredicto final

| Rol | Veredicto |
|-----|-----------|
| **PM** | El brief es buena **sugerencia de discovery**; recortado a Tier-1 + VL cases. |
| **PO** | Prioridad: documentar + scoring; providers HTTP uno a uno con approve. |
| **CTO** | Gateway separado; stub siempre; free-first; docs no sustituyen adapters. |

Siguiente fase (requiere approve): Provider SDK + un adapter real (OmniRoute image ya parcialmente listo) + correr benchmark.
