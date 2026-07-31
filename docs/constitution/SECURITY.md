# SECURITY Constitution

**Autoridad de seguridad/privacidad (Foundation 0).**

---

## 1. Principios

| # | Regla |
|---|--------|
| S1 | **Local-first** por defecto. |
| S2 | Ningún archivo sale del equipo sin **acción o política explícita**. |
| S3 | Antes de enviar a un provider: mostrar **qué datos** recibe **qué proveedor**. |
| S4 | Sin telemetría cloud obligatoria. |

---

## 2. API Keys y secretos

| # | Regla |
|---|--------|
| K1 | Desde el **primer proveedor real**: almacenamiento **seguro del sistema** (OS keychain / credential store). |
| K2 | **Nunca** guardar API keys en: código · **SQLite** · **JSON** · config en **texto plano** · **logs** · manifests/exports. |
| K3 | Stub de provider **no** requiere secrets. |
| K4 | Frontend: secretos enmascarados o no expuestos. |
| K5 | No commitear `.env` con secretos, tokens, ni DB de usuario. |

---

## 3. Filesystem y paths

| # | Regla |
|---|--------|
| F1 | Operar solo bajo directorios administrados (app data, media root, tmp de jobs). |
| F2 | Prevenir **path traversal** (`..`, roots ajenos). |
| F3 | Sanitizar nombres de archivo. |
| F4 | No seguir **symlinks** en cleanup sin validación. |
| F5 | Cleanup limitado al **workspace del job**. |
| F6 | No borrar fuera del directorio administrado. |
| F7 | No ejecutar binarios arbitrarios del usuario en generate. |
| F8 | Archivo faltante → estado controlado (no crash opaco). |

---

## 4. Logs y privacidad

| # | Regla |
|---|--------|
| L1 | Logs locales; sin phoning-home. |
| L2 | No loguear API keys, tokens, ni media en base64. |
| L3 | No loguear prompts completos con datos sensibles salvo política explícita. |
| L4 | Correlacionar con `job_id` / `request_id` / `asset_id` cuando existan. |

---

## 5. Integridad del catálogo (seguridad de producto)

| # | Regla |
|---|--------|
| P1 | No approve sin use case de Review. |
| P2 | No backdoors de approve en release. |
| P3 | Automatic solo con plan approved. |
| P4 | Generate no escribe `approved`; job generate → `waiting_review`. |
| P5 | Library no expone waiting/rejected como catálogo confiable. |

---

## 6. Operaciones destructivas

Deben mostrar antes de confirmar:

- qué se borrará
- cuántos archivos
- cuánto espacio (si se conoce)
- si es reversible

No eliminar assets automáticamente sin confirmación.

---

## 7. IPC / Tauri

| # | Regla |
|---|--------|
| I1 | Capabilities mínimos; no abrir todo el FS del usuario. |
| I2 | Validar y limitar inputs de commands. |
| I3 | Errores de UI sin filtrar secretos ni paths sensibles innecesarios. |

---

## 8. Checklist

```text
[ ] ¿Hay secrets? ¿solo OS store?
[ ] ¿Cero secrets en SQLite/JSON/plano/logs/diff?
[ ] ¿Paths validados bajo root?
[ ] ¿Cleanup solo tmp propio?
[ ] ¿Sin bypass Review/Plan?
[ ] ¿Provider disclosure si hay red?
[ ] ¿Destructivas con confirmación clara?
```

---

## 9. Referencias

- `docs/12-DECISIONS.md` D-023
- `docs/AI_PLAYBOOK.md` · `ENGINEERING.md`
