# Conectar OmniRoute (imagen + needs) — checklist e2e

**Estado adapter VL:** HTTP listo · **estado en tu máquina:** depende de que el gateway esté corriendo.

Base por defecto: `http://127.0.0.1:20128/v1`

---

## 1. Arrancar OmniRoute

Elige una forma según cómo lo instales (ver [OmniRoute](https://github.com/diegosouzapw/OmniRoute)):

```bash
# Ejemplo genérico — ajusta a tu install
omniroute
# o: docker run ... -p 20128:20128 ...
```

Comprueba:

```text
GET http://127.0.0.1:20128/v1/models
```

Debe responder JSON (lista de modelos o similar). Si no conecta → el gateway no está up.

---

## 2. Settings en Visual Library

| Campo | Valor sugerido |
|-------|----------------|
| **Providers → IA guion** | `omniroute` (needs vía Claude/chat) |
| **Providers → imagen default** | `omniroute` (o deja stub y elige omniroute en cada need) |
| **Providers → habilitados** | ☑ omniroute (+ stub siempre) |
| **Keys / Omni → Base URL** | `http://127.0.0.1:20128/v1` |
| **Image model** | **`provider/model`** (NO solo `auto`). Ej: `pollinations/flux` |
| **Chat model** | **`auto/best-free`** o `auto/chat` (NO solo `auto`) |
| **Preferir free** | ☑ |
| **API key Omni** | vacía si no la pide; o la del gateway |

Pulsa **Probar OmniRoute** en Settings (Keys). Debe decir:

- `models: ok` y/o  
- `images: ok` (si el gateway ya tiene backend de imagen)

---

## 3. Smoke de 3 prompts (bench corto)

Desde **Factory → Manual**:

1. Need con provider `omniroute`, 1 variante.  
2. Prompt (copiar uno):

| id | prompt |
|----|--------|
| `nature-tree` | Cross-section of a tree showing roots trunk leaves, educational diagram style, soft colors, no text |
| `tech-chip` | Stylized CPU chip macro illustration for tech literacy, clean edges, blue-gray palette, no text |
| `education-classroom` | Empty bright classroom with chalkboard and desks, wide shot, inviting, no readable writing |

3. Preview → Submit → **Review**: la imagen debe ser real (no tile stub de color).  
4. Si ves provider `stub` en Review: OmniRoute falló y hubo **fallback** (mira el mensaje de error o Settings → Probar).

Plantilla de resultado: copiar a `docs/providers/runs/YYYY-MM-DD-omniroute.md` (opcional).

---

## 4. Needs vía Claude (chat)

1. Settings → script AI = **omniroute**  
2. Chat model = id Claude (o `auto` si el gateway enruta)  
3. Opcional: edita **Prompt needs**  
4. Factory → Guion → **Proponer needs**  
5. Método esperado: `omniroute_chat_json_v1`  
   Si ves `fallback_heuristic_…` → chat down o JSON inválido

---

## 5. Fallos frecuentes

| Síntoma | Causa probable | Qué hacer |
|---------|----------------|-----------|
| Probar: connection refused | Gateway apagado | Arrancar OmniRoute, puerto 20128 |
| images HTTP 404 | Sin ruta `/v1/images/generations` | Habilitar image backend en OmniRoute |
| images HTTP 4xx model | Model id incorrecto | GET /v1/images/generations; elige id `pollinations/…` del menú |
| images **0 models** en Endpoints | Provider de imagen no conectado | OmniRoute → Proveedores → Pollinations |
| images **401 Authentication required** | Pollinations exige su propia key | Editar conexión Pollinations → Clave API de enter.pollinations.ai (además de la key OmniRoute en VL) |
| Generate → stub en Review | Fallback en `generate_stub_asset` | Probar OmniRoute; revisa model |
| Needs = heurística | Chat falló | Chat model + Probar; lee notes en Factory |

---

## 6. Límites de esta fase

- No hace falta otra API key de OpenAI/Stability para probar **OmniRoute**.  
- Calidad = backend que enrute el gateway (D-039).  
- Presupuesto conector `omniroute` sigue contando unidades free en Settings.
