> **LEGACY — no normativo.** Fuente de verdad: [README.md](./README.md) y Foundation 0 (`docs/AI_PLAYBOOK.md`, `PRODUCT.md`, `ARCHITECTURE.md`, `ENGINEERING.md`, `UX_UI.md`, `TESTING.md`, `SECURITY.md`, `DONE.md`). Si hay conflicto, **ignorar este archivo**.

# 04 â€” Data Constitution

**Estado:** Normativo
**Ãmbito:** SQLite, filesystem de media, migraciones, duplicados, integridad
**Realidad del repo:** diseÃ±o en `docs/05-DATABASE.md`; sin crate SQLite ni migraciones en cÃ³digo aÃºn (negocio en pausa).

Estas reglas rigen **toda** implementaciÃ³n futura de datos.

---

## 1. Dual store (canonicidad)

| Store | Es dueÃ±o de | No es dueÃ±o de |
|-------|-------------|----------------|
| **SQLite** | metadata, estados, relaciones, jobs, settings | bytes de imagen |
| **Filesystem administrado** | bytes de assets, tmp de jobs, exports | estado de review/library |

| # | Ley |
|---|-----|
| D-1 | SQLite = verdad de metadata y estados (E-7). |
| D-2 | FS = verdad de bytes (E-8). |
| D-3 | No duplicar canonicidad en memoria/JSON/FE store (E-5). |
| D-4 | No almacenar imÃ¡genes como BLOBs en SQLite salvo necesidad demostrada y aprobada. |

---

## 2. SQLite (D-025)

| # | Regla |
|---|--------|
| D-10 | Usar **migraciones numeradas** en repo (`0001_â€¦.sql`, â€¦). |
| D-11 | Toda migraciÃ³n debe ser **transaccional** (o documentar por quÃ© no y compensar). |
| D-12 | Registrar versiÃ³n de schema (`schema_migrations`). |
| D-13 | Probar desde **base vacÃ­a**. |
| D-14 | Probar desde la **Ãºltima versiÃ³n publicada** (upgrade) cuando exista release. |
| D-15 | No ignorar errores de `ALTER TABLE` / migrate. |
| D-16 | No cambios destructivos sin **backup** y **aprobaciÃ³n**. |
| D-17 | Activar y comprobar **foreign keys** (`PRAGMA foreign_keys = ON`) desde el inicio. |
| D-18 | Activar **WAL** desde el inicio (`PRAGMA journal_mode = WAL`). |
| D-19 | **Nunca modificar migraciones ya publicadas.** Solo aÃ±adir migraciones nuevas. |
| D-20 | Ãndices para consultas **reales** â€” no especulativos. |
| D-21 | Acceso parametrizado siempre. |
| D-22 | `busy_timeout` razonable en desktop. |

Stack previsto: `rusqlite` + SQL files â€” **no instalado aÃºn** (negocio en pausa).

---

## 3. Archivos (MediaStore)

| # | Regla |
|---|--------|
| D-30 | La aplicaciÃ³n administra su propio directorio (app data + media root). |
| D-31 | Todo asset adoptado: **ID**, **SHA-256**, **path**, **tamaÃ±o**, **mime type**, **dimensiones**, **estado**. |
| D-32 | **No sobrescribir** archivos de assets existentes. |
| D-33 | Rutas **temporales** para procesos en curso. |
| D-34 | Promover a definitivo **solo tras validaciÃ³n** (existencia, hash, mime/dimensiones). |
| D-35 | No borrar fuera del directorio administrado. |
| D-36 | Validar **symlinks** y **path traversal**. |
| D-37 | Archivo faltante â†’ estado controlado / issue de integridad. |
| D-38 | Conservar nombre original como metadata cuando exista. |
| D-39 | Nombres internos por **ID** o **hash**. |

---

## 4. Duplicados (D-022)

| Mecanismo | MVP | Post-MVP |
|-----------|-----|----------|
| **SHA-256** | **SÃ­** â€” duplicados exactos | â€” |
| **pHash** | **No** | Fase posterior documentada |

| # | Ley |
|---|-----|
| D-40 | MVP: **Ãºnicamente SHA-256**. |
| D-41 | pHash **fuera del MVP**; requiere fase + ADR propios. |
| D-42 | No eliminar automÃ¡ticamente assets por dedup. |
| D-43 | Coincidencias exactas pueden informar FOUND / mark duplicate; borrado solo con confirmaciÃ³n. |

---

## 5. Integridad de dominio en datos

| # | Ley |
|---|-----|
| D-50 | Library solo `approved`. |
| D-51 | Review queue = assets `waiting_review`. |
| D-52 | Soft lifecycle preferido a hard-delete. |
| D-53 | `AssetUsage.consumer` string estable â€” sin FK a VigilCut. |
| D-54 | Paths de media preferentemente relativos al media root. |

---

## 6. Backup y portabilidad

| # | Ley |
|---|-----|
| D-60 | Backup = DB (+ WAL/shm) + Ã¡rbol media. |
| D-61 | Checkpoint/cierre cuidadoso antes de copiar. |
| D-62 | No sync cloud como â€œbackupâ€. |

---

## 7. Testing de datos (cuando exista implementaciÃ³n)

```text
cargo test -p visual_library_infrastructure
```

Cada migraciÃ³n: empty Â· upgrade Â· re-run Â· FKs Â· preservaciÃ³n Â· backup si destructiva.

---

## 8. Anti-patrones

- BLOB de imagen en SQLite
- Editar migraciÃ³n publicada
- pHash en MVP
- Segunda verdad en JSON paralelo a SQLite
- Sobrescribir path final a mitad de generate

---

## 9. Checklist PR datos

- [ ] MigraciÃ³n numerada **nueva** (no editada)
- [ ] WAL + FKs
- [ ] Tests empty/upgrade
- [ ] SHA-256 en assets
- [ ] Path safety

---

## 10. Referencias

- `docs/05-DATABASE.md` Â· `docs/12-DECISIONS.md` (D-022, D-025)
- [05-JOBS-CONSTITUTION.md](./05-JOBS-CONSTITUTION.md)
