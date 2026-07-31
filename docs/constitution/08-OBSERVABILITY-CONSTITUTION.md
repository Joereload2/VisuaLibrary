> **LEGACY — no normativo.** Fuente de verdad: [README.md](./README.md) y Foundation 0 (`docs/AI_PLAYBOOK.md`, `PRODUCT.md`, `ARCHITECTURE.md`, `ENGINEERING.md`, `UX_UI.md`, `TESTING.md`, `SECURITY.md`, `DONE.md`). Si hay conflicto, **ignorar este archivo**.

# 08 Ã¢â‚¬â€ Observability Constitution

**Estado:** Normativo
**ÃƒÂmbito:** Logs, mÃƒÂ©tricas locales, diagnÃƒÂ³stico de jobs, seÃƒÂ±ales de UI
**Realidad del repo:** sin `tracing` subscriber de producto aÃƒÂºn; jobs no implementados. Las reglas aplican al diseÃƒÂ±ar e implementar.

---

## 1. Preguntas que todo proceso relevante debe poder responder

- quÃƒÂ© ocurriÃƒÂ³
- cuÃƒÂ¡ndo
- con quÃƒÂ© input
- con quÃƒÂ© configuraciÃƒÂ³n
- quÃƒÂ© proveedor
- cuÃƒÂ¡nto costÃƒÂ³ (unknown Ã¢â€°Â  0)
- quÃƒÂ© produjo
- por quÃƒÂ© fallÃƒÂ³

---

## 2. Principios

| # | Ley |
|---|-----|
| O-1 | Observabilidad por defecto **local** (logs en app data + tablas jobs/events). |
| O-2 | Sin telemetrÃƒÂ­a cloud obligatoria. |
| O-3 | Trabajo largo correlacionable por `job_id` / `request_id`. |
| O-4 | UI traduce errores a **acciones humanas**; detalle tÃƒÂ©cnico copiable y plegable. |
| O-5 | Eventos de progreso **no** son la fuente de verdad (Jobs J-9); sirven de seÃƒÂ±al. |

---

## 3. Logs

### Incluir

| Campo | Uso |
|-------|-----|
| timestamp | cuÃƒÂ¡ndo |
| level | error/warn/info/debug |
| operation | use case / handler |
| job_id | correlaciÃƒÂ³n |
| request_id | GenerationRequest u otro |
| asset_id | cuando aplique |
| error_code | estable |

### No incluir

- API keys / tokens
- imÃƒÂ¡genes en base64
- datos sensibles innecesarios
- prompts completos con secretos (salvo polÃƒÂ­tica explÃƒÂ­cita S-10)

| # | Ley |
|---|-----|
| O-10 | Preferir logging estructurado (`tracing` u otro aprobado en fase). |
| O-11 | Hoy no hay stack de logs de producto: al aÃƒÂ±adir, cumplir esta tabla desde el primer commit de esa fase. |

---

## 4. MÃƒÂ©tricas iniciales (locales)

Cuando existan contadores/consultas:

- jobs creados / completados / fallidos / reintentados
- duraciÃƒÂ³n
- coste (incl. unknown)
- assets generados / aprobados
- rechazo por motivo
- duplicados
- uso por concepto
- consultas sin resultados

| # | Ley |
|---|-----|
| O-20 | MÃƒÂ©tricas se calculan localmente; no se exportan a terceros sin opt-in post-MVP. |
| O-21 | Contadores solo en memoria no sustituyen jobs ni coverage. |

---

## 5. DiagnÃƒÂ³stico de producto (UI)

| SeÃƒÂ±al | EstaciÃƒÂ³n / widget |
|-------|-------------------|
| Waiting Review count | badge Review |
| Jobs failed / interrupted | Factory o Settings (no 7Ã‚Âº flujo) |
| Coverage issues | Coverage |
| Missing file integrity | Coverage / health |
| Queued prolongado | explicaciÃƒÂ³n diagnÃƒÂ³stica (J-12) |

La UI debe mapear `error_code` Ã¢â€ â€™ `suggested_action` (contrato de error FE).

---

## 6. Domain events (opcional)

| # | Ley |
|---|-----|
| O-30 | Tabla `domain_events` opcional para auditorÃƒÂ­a/tests. |
| O-31 | No bus distribuido / Kafka / outbox cloud en MVP. |

---

## 7. Anti-patrones

- Solo `println!` en release
- TelemetrÃƒÂ­a phoning-home
- Log de secretos
- Ã¢â‚¬Å“Completed 0%Ã¢â‚¬Â
- Silenciar fallos de job

---

## 8. Checklist PR

- [ ] Ã‚Â¿Se puede responder el Ã‚Â§1 para el flujo tocado?
- [ ] Ã‚Â¿CorrelaciÃƒÂ³n job/request/asset?
- [ ] Ã‚Â¿Logs sin secretos?
- [ ] Ã‚Â¿Error Ã¢â€ â€™ acciÃƒÂ³n humana en UI?

---

## 9. Referencias

- `docs/06-JOBS.md`
- [05-JOBS-CONSTITUTION.md](./05-JOBS-CONSTITUTION.md) Ã‚Â· [07-SECURITY-PRIVACY-CONSTITUTION.md](./07-SECURITY-PRIVACY-CONSTITUTION.md)
