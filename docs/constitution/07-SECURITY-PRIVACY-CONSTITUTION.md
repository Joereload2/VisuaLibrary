> **LEGACY — no normativo.** Fuente de verdad: [README.md](./README.md) y Foundation 0 (`docs/AI_PLAYBOOK.md`, `PRODUCT.md`, `ARCHITECTURE.md`, `ENGINEERING.md`, `UX_UI.md`, `TESTING.md`, `SECURITY.md`, `DONE.md`). Si hay conflicto, **ignorar este archivo**.

# 07 â€” Security & Privacy Constitution

**Estado:** Normativo
**Ãmbito:** Datos locales, secrets, filesystem, IPC, providers
**Contexto:** app desktop Tauri local-first; un usuario de mÃ¡quina; sin auth cloud en MVP

---

## 1. Principios

| # | Regla |
|---|--------|
| S-1 | **Local-first** por defecto. |
| S-2 | NingÃºn archivo sale del equipo sin acciÃ³n o polÃ­tica explÃ­cita del usuario. |
| S-3 | Mostrar quÃ© proveedor recibirÃ¡ quÃ© datos antes de generar/enviar. |
| S-4 | API keys **nunca** en: cÃ³digo Â· SQLite Â· JSON Â· configuraciÃ³n en texto plano Â· logs Â· manifests/exports (D-023). |
| S-5 | Desde el **primer proveedor real**, usar **almacenamiento seguro del sistema** (keychain/credential store). El stub no requiere keys. |
| S-6 | Sanitizar nombres y rutas. |
| S-7 | Prevenir path traversal. |
| S-8 | No seguir symlinks durante limpieza sin validaciÃ³n. |
| S-9 | Limitar limpieza al workspace del job (tmp propio). |
| S-10 | No registrar prompts completos si contienen informaciÃ³n sensible, salvo polÃ­tica explÃ­cita. |
| S-11 | No eliminar assets automÃ¡ticamente sin confirmaciÃ³n. |
| S-12 | Registrar proveedor, modelo, coste y procedencia (coste unknown â‰  0). |
| S-13 | Registrar derechos/licencia cuando se conozcan. |
| S-14 | Operaciones destructivas: quÃ© se borra Â· cuÃ¡ntos archivos Â· cuÃ¡nto espacio Â· si es reversible. |

---

## 2. Integridad del catÃ¡logo

| # | Ley |
|---|-----|
| S-20 | No hay command que apruebe sin use case de Review. |
| S-21 | No backdoors `dev_approve_all` en release. |
| S-22 | Automatic no ejecuta planes no `approved`. |
| S-23 | Library no presenta waiting_review/rejected como catÃ¡logo confiable. |
| S-24 | Generate no escribe `approved`. |
| S-25 | Job de generate termina en `waiting_review` (D-019), no aprueba. |

Un bypass es vulnerabilidad de producto.

---

## 3. Secrets e IPC (Tauri)

| # | Ley |
|---|-----|
| S-30 | Frontend: secrets enmascarados o nunca. |
| S-31 | Capabilities Tauri mÃ­nimos. |
| S-32 | Validar inputs de commands. |
| S-33 | Errores de UI no filtran secrets. |
| S-34 | `.env`, key files y DB de usuario en `.gitignore`. |

---

## 4. Filesystem

| # | Ley |
|---|-----|
| S-40 | Media root + app data administrados. |
| S-41 | Paths bajo root; rechazar `..`. |
| S-42 | Tmp de jobs aislado. |
| S-43 | No ejecutar binarios arbitrarios del usuario en generate. |
| S-44 | Archivo faltante = estado controlado. |

---

## 5. Red y providers

| # | Ley |
|---|-----|
| S-50 | Red solo a providers habilitados. |
| S-51 | Timeouts y lÃ­mites de reintento. |
| S-52 | Stub sin red en dev/tests. |
| S-53 | Sin telemetrÃ­a cloud obligatoria. |

---

## 6. Repo

| # | Ley |
|---|-----|
| S-60 | No commitear `target/`, `node_modules/`, `*.sqlite`, media real, tokens. |
| S-61 | Dependencias nuevas solo con necesidad demostrada y aprobaciÃ³n. |

---

## 7. Checklist PR

- [ ] Â¿Secrets? Â¿OS secure store?
- [ ] Â¿Path rules?
- [ ] Â¿Bypass Review/Plan?
- [ ] Â¿Logs limpios?

---

## 8. Referencias

- `docs/12-DECISIONS.md` (D-023)
- [04-DATA-CONSTITUTION.md](./04-DATA-CONSTITUTION.md)
