# Visual Library

Aplicación de escritorio **local** para crear, organizar, revisar y consultar recursos visuales reutilizables centrados en **conceptos**.

Producto independiente (no es un módulo de VigilCut). Stack: **Tauri 2 + Rust + React + SQLite**.

## Documentación

Empieza aquí: [`docs/00-START-HERE.md`](docs/00-START-HERE.md)

## Estado

| Fase | Estado |
|------|--------|
| Foundation 0 (metodología + diseño) | **Aprobada** |
| Foundation 1 (SQLite + settings + paths) | **Hecha** |
| Foundation 2 (domain + catálogo ensure/list) | **Hecha** |
| Foundation 3 (generate stub + Review + Library assets) | **Hecha** |
| Foundation 4 (Manual Factory FOUND/GENERATE) | **Hecha** |
| Foundation 5 (Plans + Automatic Factory) | **Hecha** |
| Foundation 6 (Coverage + Review completo) | **Hecha** |
| OmniRoute + Manual Factory real (imagen/chat) | **Lista** (gateway + keys/pollen) |
| Catálogo providers + quality gate | **Hecha** (docs + `pnpm quality`) |
| Scaffold monorepo | Hecha |
| App alimentadora de guiones | **Siguiente foco de producto** (fuera de este repo) — ver [`docs/SCRIPT-FEEDER-HANDOFF.md`](docs/SCRIPT-FEEDER-HANDOFF.md) |
| Providers volumen (Fal/…) / import archivo guion | Pendiente al retomar VL |
| Job recovery / Library search-export / polish | Pendiente (post-MVP UI) |

## Estructura

```
apps/desktop/          # Shell Tauri
packages/ui/           # Frontend (6 flujos)
crates/domain/         # Dominio puro
crates/application/    # Casos de uso + ports
crates/infrastructure/ # Adapters (SQLite/FS/jobs más adelante)
docs/                  # Fundación del producto
```

## Requisitos

- Rust (stable) + Cargo
- Node.js 20+
- pnpm 8+
- [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) (Windows: WebView2, MSVC)

## Comandos

```bash
# Instalar JS
pnpm install
pnpm test:e2e:install   # browsers Playwright (una vez)

# Calidad de código (docs/reglas-calidad-codigo.md)
pnpm quality              # fmt check + clippy + tests Rust + tsc + vitest
pnpm quality:strict       # clippy -D warnings
pnpm quality:ps1          # mismo gate vía PowerShell
pnpm fmt:rust
pnpm clippy
pnpm check:ui             # tsc --noEmit
pnpm test:rust
pnpm test:ui

# E2E UI (Playwright sobre Vite — no Tauri completo)
pnpm test:e2e

# Suite tests
pnpm test

# App desktop (dev)
pnpm dev

# Solo UI en browser
pnpm dev:ui
```

**MVP usable (F1–F6):** las 6 estaciones tienen flujo real. Generación: `stub` local o **OmniRoute** (OpenAI-compatible) cuando el gateway y las keys están conectados. Entrada de guion hoy: paste en Manual Factory — contrato para la app hermana en el handoff.

## Navegación MVP (6 flujos)

Factory · Review · Library · Coverage · Plans · Settings

Sin pantallas CRUD de Conceptos / Representaciones / Assets.

### Cómo probar la interfaz

```bash
pnpm dev          # app desktop Tauri (IPC real)
# o solo UI en browser (IPC no disponible):
pnpm dev:ui
```

Recorrido sugerido: **Settings** (paths) → **Library** (ensure concept + generate stub) → **Review** (approve / edit / regenerate / duplicate) → **Factory** (manual o automatic con plan) → **Plans** → **Coverage** (issues con CTA).
