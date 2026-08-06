# Production Package — path compartido

**Default:** `%USERPROFILE%\Documents\FacelessStudio\packages\{package_id}\`

| Archivo | Quién escribe |
|---------|----------------|
| `package.yaml` | YouToMagic (script + DNA); VL actualiza needs/assets; FC audio/timeline |
| `script.md` | YouToMagic (legacy paste) |
| `media/images/` | VisuaLibrary (approved) |
| `media/audio/` | FacelessCreator (ElevenLabs) |
| `export/` | FacelessCreator (draft.mp4) |
| `events.jsonl` | Todas (append) |

**Env override:** `FACELESS_STUDIO_PACKAGES`

## Estado infra VL

- Contrato documentado.
- Import UI completo: pendiente (próximo cuando se retome VL).
- Manual Factory sigue aceptando **paste** de `script.md` / cuerpo del guion.
- Al generar, copiar PNG a `media/images/{beat_id}.png` cuando exista `beat_id` en metadata.

## Mapa de apps

Ver `YouToMagic/docs/18-ECOSISTEMA-APPS.md`.
