# Modo packaging (miniaturas con texto)

**Estado 2026-08-06:** cableado del contrato en el ecosistema; generación real de thumbs en **FacelessCreator** (`/api/packages/thumbs`) con:

| Provider | Credencial | Sin key |
|----------|------------|---------|
| Stub local | — | escribe `media/thumbs/*.stub.txt` + prompt |
| OpenAI Images | `OPENAI_API_KEY` | error → fallback stub |
| Gemini Imagen | `GEMINI_API_KEY` | skeleton (NotImplemented con key hasta confirmar endpoint) |

## Política

| Tipo asset | Texto en frame | App | Modelo típico |
|------------|----------------|-----|----------------|
| Lección (Review) | **Prohibido** | VisuaLibrary | OmniRoute / flux barato |
| Miniatura A/B | **Permitido** (≤5 palabras) | FC packaging o VL packaging | Gemini / GPT-image / Ideogram |

## Contrato en package

```json
"packaging": {
  "titles": [{"variant_id":"t1","text":"...","hypothesis":"base"}],
  "thumbnails": [{
    "variant_id": "th1",
    "text": "EL ERROR",
    "hypothesis": "texto_corto_dolor",
    "path": "media/thumbs/th1.png",
    "image_provider": "openai",
    "image_model": "gpt-image-1"
  }]
}
```

YouToMagic al crear el lote ya deja **textos de thumb + hipótesis** en `brief.yaml` y `packaging_variants` (sin bytes).  
FC/VL rellenan `path` + provider/model → YTM **Sincronizar medicion** importa para rankings.

## VL (lección)

1. Importar `package.yaml` / beats con `text_in_image=forbidden`.  
2. Review → copiar PNG a `media/images/` del episodio.  
3. Opcional: set meta `lesson_image_provider` / `lesson_image_model` en package para atribución.

Hasta UI packaging nativa en VL, usar **FC → Miniaturas**.
