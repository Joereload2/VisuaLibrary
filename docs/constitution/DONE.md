# DONE — Checklist

**Foundation 0** · Checklist. No explicación.
Si un ítem **aplicable** falla → **no Done**.

Marcar `N/A` solo con justificación en la entrega.

---

## Producto

```text
[ ] Objetivo de la TASK CARD cumplido
[ ] Alcance respetado (sin ampliar)
[ ] No objetivos respetados
[ ] Criterios de aceptación verificados
[ ] Invariantes de producto intactas (Review gate, plan approved, 6 flujos)
```

---

## Arquitectura

```text
[ ] Capas respetadas (domain sin I/O; UI sin SQL)
[ ] Sin acoplamiento VigilCut
[ ] Sin OmniRoute / deps no aprobadas
[ ] Tarea no debió dividirse (o ya se dividió)
[ ] Orden de implementación respetado (UI no primero en feature nueva)
```

---

## Frontend

```text
[ ] Código en flows/ correctos
[ ] Contratos tipados; sin any permanente
[ ] Sin canonicidad solo en React
[ ] Anti doble envío
[ ] pnpm test:ui (si tocó UI)
[ ] pnpm build:ui (si tocó UI)
[ ] pnpm test:e2e (si riesgo HIGH/ARCHITECTURE y UI de flujo)
```

---

## Backend

```text
[ ] Use case de producto (no set_status)
[ ] Invariantes en domain/application
[ ] Errores estructurados
[ ] Transacciones si multi-write
[ ] Idempotencia si reintento
[ ] Sin unwrap/expect/panic productivos nuevos
[ ] pnpm fmt:rust + check:rust + test:rust (si tocó Rust)
```

---

## SQLite / datos

```text
[ ] Migración nueva numerada (si schema)
[ ] No se editó migración publicada
[ ] WAL + FKs (si toca open/migrate)
[ ] Sin secrets en DB
[ ] SHA-256 only (no pHash en MVP)
```

---

## Jobs

```text
[ ] Persistido antes de run
[ ] Generate → waiting_review (no completed)
[ ] Cancel/retry/recovery considerados
[ ] Progress no miente (no completed engañoso)
[ ] Cleanup solo tmp propio
```

---

## UX / UI

```text
[ ] Checklist UX_UI.md §7
[ ] Empty/loading/error/success
[ ] Sin overflow horizontal
[ ] Acción primaria única
[ ] Waiting Review no presentado como Library
[ ] Screenshots (si UI real de feature)
```

---

## Seguridad

```text
[ ] Checklist SECURITY.md §8
[ ] Sin secrets en diff
```

---

## QA / pruebas

```text
[ ] Pruebas del nivel de riesgo ejecutadas (TESTING.md §4)
[ ] Unit / Integration / Smoke / E2E / Regression según aplique
[ ] Fallidas = 0
[ ] Omitidas listadas con motivo
[ ] git diff --check OK
```

---

## Documentación

```text
[ ] Docs/ADR actualizados si cambió comportamiento o decisión
[ ] No se alteraron constituciones para eludir reglas
[ ] Sin instrucciones obsoletas introducidas
```

---

## Entrega

```text
[ ] Resumen
[ ] Archivos modificados
[ ] Pruebas ejecutadas / omitidas
[ ] Problemas y riesgos
[ ] Capturas si aplica
[ ] git diff --check
[ ] Sin commit/push salvo autorización
[ ] Loop errores + loop simplificar hechos
[ ] Revisión multi-rol solo si HIGH/ARCHITECTURE (playbook)
```


---

## Bloqueo duro (automático no-Done)

```text
[ ] Library sin Approve
[ ] Generate job completed en vez de waiting_review
[ ] Plan draft genera
[ ] Secrets en SQLite/JSON/logs
[ ] Jobs solo en memoria
[ ] VigilCut en core
[ ] Placeholder como funcionalidad
[ ] Tests obligatorios fallidos
```

---

## Referencia

`docs/AI_PLAYBOOK.md` · constituciones hermanas
