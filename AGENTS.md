# AGENTS.md — VisuaLibrary (Visual Library)

Fuente de verdad para agentes en este repo.

## Rol en el ecosistema (no negociable)

Producto **independiente** de librería visual por **conceptos**. No es módulo de VigilCut.

| Hace | No hace |
|------|---------|
| Factory / Review / Library / Coverage / Plans | Research de nichos (YouToMagic) |
| Needs desde guion o **package** FacelessStudio | Guion final / ElevenLabs |
| Generate (stub u OmniRoute) + gate Review | FFmpeg / timeline / MP4 |
| Write-back a `media/images/{beat_id}.*` del package | Shorts (VigilCut) |

Flujo package: Manual Factory → **Abrir package** → needs → Submit → **Review (Approve)**.

Al **Approve** en Review, si el asset tiene `package_path` + `beat_id` (generado desde package), se hace **write-back automático** a `media/images/{beat_id}.*`. El botón manual «Escribir al package» en Factory sigue disponible para batch.

Package root: `%USERPROFILE%\Documents\FacelessStudio\packages\`  
(`FACELESS_STUDIO_PACKAGES` override)

## Stack

- Tauri 2 + Rust (domain / application / infrastructure)
- Frontend: React + Vite en `packages/ui`
- SQLite + media root local
- Monorepo: `crates/*`, `apps/desktop`, `packages/ui`

## Comandos

```bash
pnpm install
pnpm quality          # fmt + clippy + tests Rust + tsc + vitest
pnpm test
pnpm dev              # desktop Tauri
pnpm dev:ui           # solo UI browser
```

Rust only:

```bash
cargo test -p visual_library_application
cargo check -p visual_library_desktop
```

## Package handoff (código)

- Módulo: `crates/application/src/package/`
- Commands Tauri: `list_packages_cmd`, `load_package_detail_cmd`, `propose_needs_from_package_cmd`, `write_package_images_cmd`, `approve_asset_cmd` (con write-back)
- Assets llevan proveniencia opcional: `package_id`, `package_path`, `beat_id`, `package_concept_key` (migración 0003)
- `package.yaml` en disco = **JSON** (schema 0.1 en `Documents/FacelessStudio/schemas/package.schema.json`)
- `validate_package_shape` al cargar package (nivel import)

## Docs clave

- `docs/00-START-HERE.md`, `docs/01-PRODUCT.md`
- `docs/SCRIPT-FEEDER-HANDOFF.md`
- YouToMagic `docs/18-ECOSISTEMA-APPS.md` (canónico de roles)

## Al cambiar código

- Ninguna imagen entra a Library sin Review.
- No generar guiones ni TTS aquí.
- Actualizar este archivo si cambian estaciones, package API o comandos.
