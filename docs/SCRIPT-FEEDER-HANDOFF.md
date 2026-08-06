# Handoff — App alimentadora de guiones → Visual Library

**Estado:** Visual Library MVP (F0–F6) + OmniRoute/imagen real está **congelado listo** para construir la app hermana que produce guiones.

Esta nota es el contrato de integración **desde la app de guiones hacia VL**. No implementa el bridge todavía.

---

## Roles de producto

| App | Responsabilidad |
|-----|-----------------|
| **Script feeder** (por construir) | Idea → guion de lección/YouTube (texto limpio, estructura didáctica). |
| **Visual Library** (este repo) | Guion → needs visuales → generate → Review → Library approved. |

VL **no** escribe guiones. El feeder **no** genera ni curate la librería de imágenes.

---

## Mapa de apps (2026-08-06)

| App | Rol |
|-----|-----|
| **YouToMagic** | Exporta guion approved + `package.yaml` a `Documents/FacelessStudio/packages/` |
| **VisuaLibrary** | Importa guion/package → needs → Review → `media/images/` |
| **FacelessCreator** | Importa package → TTS + montaje long |
| **VigilCut** | Shorts |

Ver también: `PACKAGE-PATH.md`, `PRODUCTION-PACKAGE.md`, maestro en YTM `docs/18-ECOSISTEMA-APPS.md`.

---

## Contrato de entrada actual (v1)

Hoy Manual Factory acepta **texto plano de guion** (paste en UI). También puede usarse `script.md` del package exportado por YouToMagic.

| Campo | Tipo | Reglas |
|-------|------|--------|
| `script` | `string` | Requerido. Trim; no vacío; **mín. ~20 caracteres**. |
| `max_needs` | `number?` | Opcional. Default **8**, clamp **1–20**. |

### Comando Tauri / caso de uso

```text
propose_needs_from_script({ script, max_needs? })
  → { needs[], script_instructions, method, notes }
```

- `needs[]` = filas editables (requerimientos BD + prompt + excerpt + variantes).
- `script_instructions` = brief global editable (no va a Library solo).
- Humano **siempre** revisa needs antes de submit generate.
- Generate → `waiting_review` → solo `approved` entra a Library.

Providers: Settings (`omniroute` / stub / …). Free-first en Automatic cuando aplica.

### Forma útil de un need (salida, no entrada del feeder)

```json
{
  "concept_key": "water-cycle",
  "concept_name": "Water cycle",
  "representation_key": "lesson",
  "prompt": "…",
  "orientation": "landscape",
  "style": "didactic",
  "script_excerpt": "tramo del guion",
  "ai_instructions": "…",
  "pedagogical_intent": "…",
  "variant_count": 3,
  "included": true
}
```

Política de imagen en VL: **sin texto/letras en el frame** (guard en prompts).

---

## Qué debe producir la app de guiones (v1)

Mínimo viable para alimentar VL **sin código de bridge**:

1. **Un archivo o clipboard de texto** `.txt` / `.md` con el guion final de la lección.
2. Guion en **idioma del video** (p. ej. español), listo para lección (no outline crudo).
3. Longitud suficiente para partir en tramos (≥ unas frases; ideal 300–3000+ palabras según lección).
4. Opcional: metadatos en cabecera YAML/frontmatter **ignorables** por VL hoy:

```markdown
---
title: Ciclo del agua
audience: secundaria
locale: es
---

Cuerpo del guion hablado…
```

VL v1 solo usa el **cuerpo de texto** pegado en Manual Factory.

---

## Bridge futuro (no implementado)

Cuando ambas apps existan, candidatos en orden de simplicidad:

| Opción | Descripción |
|--------|-------------|
| **A. Archivo + paste** | Feeder exporta `.txt`; usuario pega en VL. Zero integración. **Ahora.** |
| **B. Drop zone / open file** | VL lee `.txt`/`.md` local en Factory. |
| **C. Carpeta compartida** | `~/VisuaLibrary/inbox/scripts/*.md` + botón “Importar último”. |
| **D. IPC / deep link** | `visuallibrary://import-script?path=…` o puerto local. |
| **E. Needs pre-estructurados** | Feeder emite JSON de needs; VL salta propose (riesgo: saltarse review humana — no por defecto). |

**Recomendación al arrancar el feeder:** diseñar export **texto limpio + frontmatter opcional** (A/B). No acoplar al JSON de needs hasta que el propose de VL sea estable en producción.

---

## Qué NO hacer en el feeder

- No generar imágenes ni tokens de providers de VL.
- No escribir en la SQLite de VL.
- No asumir multi-usuario cloud.
- No meter marcas de watermark/texto en el guion pensando que saldrán en imagen (VL pide lo contrario en el visual).

---

## Estado de VL al handoff

| Área | Estado |
|------|--------|
| F0–F6 flujos locales | Hechos |
| Manual Factory guion → needs → generate | Hecho |
| OmniRoute image + chat | Listo; requiere gateway + pollen/key |
| Catálogo providers / scoring (docs) | Hecho (D-039) |
| Quality gate `pnpm quality` | Hecho (D-040) |
| Adapter Fal/Pollinations directo | Pendiente (siguiente cuando se retome VL) |
| Import file / inbox de guiones | No |

Comandos útiles: `pnpm dev`, `pnpm quality`. Docs: `docs/00-START-HERE.md`, providers en `docs/providers/`.

---

## Checklist al volver a VL

1. Probar un guion real exportado por el feeder (paste → propose → submit → Review).  
2. Si duele el paste: implementar **B** (open file).  
3. Provider de volumen estable si pollen no alcanza.  
4. Solo entonces valorar JSON needs o deep link.
