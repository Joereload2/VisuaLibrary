# Production Package — contrato compartido (VisuaLibrary)

**Canónico extendido:** `YouToMagic/docs/15-pipeline-produccion-unificado.md`

Este archivo resume el encaje de **VisuaLibrary** en el pipeline:

> Guion aprobado (beats) → needs/prompts en el **mismo idioma** (Channel DNA) → generate → Review → assets en el package → (fuera de VL) TTS ElevenLabs + ensamblado.

## Qué es el “mismo idioma”

| Concepto | Uso en VL |
|----------|-----------|
| `channel_dna` | Inyectar style, palette, forbidden (no text) en generate |
| `beat` | Trazar need ↔ fragmento de guion (`script_excerpt`, `beat_id`) |
| `concept_key` / `representation_key` | Matching FOUND / catálogo |
| `prompt_version` | Builder único `pp-prompt-v1` (no prompts ad hoc por pantalla) |
| `package_id` | Metadata en asset/job para round-trip y analítica |

## Qué hace VL

1. Importar o proponer `image_needs` desde el package / guion.  
2. Generar con providers (OmniRoute, etc.).  
3. Review humano → solo `approved` a Library y de vuelta al package (`asset_ids`).  

## Qué no hace VL

- ElevenLabs / audio  
- Montaje FFmpeg / timeline  
- Upload YouTube  

## Telemetría mínima en VL

Eventos: `need_proposed`, `generate_started/finished/failed`, `review_approved/rejected/regen`, coste, latencia, provider, `package_id`, `beat_id`.

## Evolución desde el handoff v1

| Hoy (`SCRIPT-FEEDER-HANDOFF`) | Mañana (PP v2) |
|-------------------------------|----------------|
| Texto plano de guion | Beats + DNA + needs estructurados |
| Paste manual | Import carpeta package |
| Prompts sueltos | `pp-prompt-v1` + DNA.forbidden |

Ver documento canónico en YouToMagic para schema completo, fases y ElevenLabs.
