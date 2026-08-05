# 13 — NON-GOALS

## 1. Propósito

Definir con precisión **qué no se construye** en el MVP ni en la fase de fundación.  
Sirve como filtro anti-scope-creep para humanos y agentes.

Si un cambio cae aquí, se rechaza o se agenda **explícitamente** como post-MVP.

---

## 2. Non-goals de producto

| Non-goal | Por qué |
|----------|---------|
| Ser un módulo de VigilCut | Debe ser producto independiente |
| Depender de VigilCut para arrancar o persistir | Invierte la frontera de ownership |
| Editor de video / timeline | Es biblioteca conceptual, no NLE |
| DAM genérico enterprise | Scope infinito; nos centramos en 6 flujos |
| Red social / marketplace de assets | Fuera de local-first MVP |
| Multi-usuario concurrente con auth cloud | Usuario local único en MVP |
| Colaboración en tiempo real | Complejidad ajena al núcleo |
| Mobile apps | Desktop local primero |
| SaaS multi-tenant | Viola local-first |

---

## 3. Non-goals de alcance de flujos

| Non-goal | Por qué |
|----------|---------|
| Séptimo flujo de navegación primaria | MVP cerrado en 6 |
| Navegación primaria “Conceptos / Representaciones / Assets” | Entidades internas, no estaciones |
| “Prompt playground” libre sin schema de necesidad | Contamina Factory Manual |
| Generación aleatoria sin plan ni lista | Prohibido por filosofía Automatic/Manual |
| Aprobar en Factory (skip Review) | Viola Library gate |
| Generar dentro de Library | Viola responsabilidades |
| Ejecutar providers desde Plans | Plans = qué, no cómo |
| Producción dentro de Settings | Settings = config |
| Coverage solo como dashboard de charts | Debe ser accionable |
| Catálogo de 40+ providers **en código/runtime** | Research en `docs/providers/`; runtime = Tier 0 (D-039) |
| Tratar OmniRoute como “el modelo de imagen” | Es **gateway**; calidad = backend enrutado |
| Provider SDK multi-vendor de un golpe | Un adapter por approve; stub + omniroute primero |

---

## 4. Non-goals técnicos

| Non-goal | Por qué |
|----------|---------|
| Supabase | Nube / no local-first |
| PostgreSQL como DB del producto | Overkill; SQLite basta |
| MySQL / Mongo / Firebase | Fuera de stack |
| Backend HTTP cloud obligatorio | App local |
| Electron (como dirección) | Tauri elegido |
| Microservicios | Un proceso desktop |
| Kubernetes / Docker como runtime del usuario final | No aplica |
| Sync multi-dispositivo en MVP | Complejidad y conflicto de verdad |
| Blobs de imagen dentro de SQLite | FS administrado |
| OmniRoute | Explicitamente fuera |
| Implementar IA de generación real en fundación | Stub primero |
| Agentes autónomos multi-paso | Fuera de MVP |
| Entrenamiento de modelos locales | Fuera |
| Blockchain / NFT de assets | Irrelevante |
| Telemetría cloud obligatoria | Privacidad local |

---

## 5. Non-goals de dominio (por ahora)

| Non-goal | Notas |
|----------|-------|
| Versionado rico tipo Git de cada asset | Post-MVP posible |
| Rights management / licencias complejas | No en MVP |
| OCR / embedding search semántico | Post; FTS básico después si hace falta |
| Auto-tagging ML | No |
| Grafos de conocimiento UI completa | Relations mínimas; sin explorer dedicado como flujo |
| Import masivo histórico de carpetas caóticas | Post-MVP (P1) |
| Deduplicación perceptual (pHash) | **Post-MVP explícito** (D-022); MVP solo SHA-256 |
| Localización completa multi-idioma día 1 | es-ES first aceptable |

---

## 6. Non-goals de implementación en *esta* entrega

| Non-goal | Estado |
|----------|--------|
| Código de negocio | No |
| Migraciones aplicadas en runtime de usuario | No |
| Pantallas implementadas | No |
| Commits automáticos | No (no se hacen en esta tarea) |
| Push a GitHub | No |
| Mover estructura del repo a monorepo | No todavía (solo propuesto) |
| Integración VigilCut | No |
| Provider real de imágenes | No |

---

## 7. Anti-patrones a rechazar en code review futuro

1. `pages/Concepts.tsx` en nav primaria.  
2. `if consumer == "vigilcut" { … }` en `domain`.  
3. `asset.status = Approved` dentro del job generate.  
4. `run_automatic(plan)` sin chequear `approved`.  
5. Cola de jobs solo en `useState`.  
6. SQL desde el frontend.  
7. Paths de media sin validar root.  
8. “Quick win” de OmniRoute “por si acaso”.  
9. Añadir flujo 7 “Admin”.  
10. Features no listadas en el plan sin actualizar docs.

---

## 8. Qué sí es goal (recordatorio breve)

Para no leer este doc como vacío:

- 6 flujos MVP  
- Dominio conceptual sólido  
- SQLite + FS local  
- Jobs durables  
- Factory Manual + Automatic (stub)  
- Review gate  
- Library approved-only  
- Coverage accionable  
- Plans = qué  

Detalle: [01-PRODUCT.md](./01-PRODUCT.md).

---

## 9. Proceso para promover un non-goal a goal

1. Escribir propuesta (problema, por qué ahora).  
2. Actualizar PRODUCT + NON_GOALS + DECISIONS + IMPLEMENTATION_PLAN.  
3. Aprobación explícita.  
4. Recién entonces fase de implementación.

Sin ese proceso, **no se implementa**.

---

## 10. Referencias

- Product: [01-PRODUCT.md](./01-PRODUCT.md)  
- Decisions: [12-DECISIONS.md](./12-DECISIONS.md)  
- Plan: [11-IMPLEMENTATION_PLAN.md](./11-IMPLEMENTATION_PLAN.md)
