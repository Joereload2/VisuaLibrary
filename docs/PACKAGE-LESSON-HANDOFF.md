# Handoff lección → package (sin API extra)

Tras Review en VisuaLibrary:

1. Copia los PNG aprobados a:
   `Documents/FacelessStudio/channels/{canal}/batches/{batch}/episodes/{ep}/media/images/`
   (o al espejo `packages/{package_id}/media/images/`).
2. En FacelessCreator: elige el package → (opcional) llama readiness:
   `POST /api/packages/refresh-readiness` con `{ "package_path": "..." }`.
3. En YouToMagic: **Sincronizar medicion** para registrar eventos.
4. Opcional en `package.yaml` meta (atribución):

```json
"meta": {
  "lesson_image_provider": "omniroute",
  "lesson_image_model": "flux"
}
```

OmniRoute para generar lecciones ya está en VL Settings (credencial propia de VL).
