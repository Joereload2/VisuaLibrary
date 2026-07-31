# 01 — PRODUCT

> **Referencia ampliada — no normativa.** Autoridad de producto: [`PRODUCT.md`](./PRODUCT.md).

## 1. Visión

**Visual Library** es una aplicación de escritorio **local** para crear, organizar, revisar y consultar **recursos visuales reutilizables** centrados en **conceptos**, no en archivos sueltos.

Es un **producto independiente**. No es un módulo de VigilCut.
La primera consumidora prevista es VigilCut, pero **Visual Library nunca dependerá de VigilCut**.

---

## 2. Problema que resuelve

Las aplicaciones creativas acumulan imágenes sin modelo:

- Duplicados conceptuales
- Huecos de cobertura (falta un concepto o una representación)
- Generación ad-hoc sin revisión
- Imposible saber “qué falta” de forma accionable
- Acoplamiento de assets a un solo producto

Visual Library introduce una **biblioteca visual reutilizable** con:

1. Modelo conceptual explícito
2. Generación controlada (Factory)
3. Gate de calidad (Review)
4. Catálogo confiable (Library)
5. Diagnóstico de cobertura (Coverage)
6. Crecimiento planificado (Plans)

---

## 3. Filosofía de producto

### 3.1 El centro es el Concepto

```
Concepto  →  Representaciones  →  Assets  →  Uso
```

| Nivel | Significado |
|-------|-------------|
| **Concepto** | Idea reutilizable del dominio visual (qué se necesita expresar). |
| **Representación** | Forma concreta de expresar un concepto (ángulo semántico / visual). |
| **Asset** | Materialización binaria + metadata de una representación. |
| **Uso** | Registro de consumo del asset por apps externas o internas. |

**Consecuencia de diseño:** no se diseña la arquitectura “como un editor de video”. Se diseña como una **biblioteca de conceptos visuales**.

### 3.2 Calidad antes que volumen

Ninguna imagen generada entra a Library sin Review.

```
Generate → Waiting Review → (Approve) → Library
                         → (Reject / Duplicate / …)
```

### 3.3 Plan ≠ Factory

| Componente | Decide |
|------------|--------|
| **Plans** | **QUÉ** generar (gaps, temas, prioridades) |
| **Factory** | **CÓMO** generarlo (manual batch o automático desde plan) |

Nunca mezclar responsabilidades.

### 3.4 Flujos, no tablas

La UI y la navegación se organizan por **estaciones de trabajo** (flujos).
Concept, Representation, Asset son **entidades internas**, no ítems de menú.

### 3.5 Local-first

- Datos en máquina del usuario
- SQLite + filesystem administrado
- Sin dependencia de nube para el núcleo del producto

---

## 4. Usuarios y roles (MVP)

| Rol | Necesidad principal |
|-----|---------------------|
| **Productor visual** | Cargar necesidades y generar solo lo faltante (Manual Factory). |
| **Curador / revisor** | Aprobar, rechazar, regenerar, marcar duplicados (Review). |
| **Bibliotecario / consumidor** | Buscar y exportar assets aprobados (Library). |
| **Planificador de cobertura** | Ver gaps y crear planes de crecimiento (Coverage + Plans). |
| **Administrador local** | Rutas, proveedores, preferencias (Settings). |

En MVP un solo usuario local puede ejercer todos los roles. No hay multi-usuario ni auth cloud.

---

## 5. MVP — alcance exacto

### 5.1 Seis flujos (y solo seis)

1. **Factory** — Manual Factory + Automatic Factory
2. **Review**
3. **Library**
4. **Coverage**
5. **Plans**
6. **Settings**

Cualquier otra estación queda **fuera del MVP** (ver [13-NON_GOALS.md](./13-NON_GOALS.md)).

### 5.2 Factory

#### Manual Factory (producción)

**Entrada:** lista estructurada de necesidades visuales.
Cada necesidad incluye al menos:

- concepto
- representación
- prompt
- orientación
- estilo
- proveedor

**Comportamiento:**

1. Buscar si ya existe un recurso **suficientemente bueno**.
2. Si existe → marcar **FOUND**.
3. Si no → marcar **GENERATE**.
4. Generar **solo** los faltantes.
5. Toda imagen nueva termina en **Waiting Review**.
6. **Nunca** pasa directo a Library.

#### Automatic Factory (crecimiento)

**Entrada canónica:**

```
Theme → Plan (aprobado) → Conceptos → Representaciones
      → Solicitudes de generación → Waiting Review
```

- No genera imágenes aleatorias.
- Solo opera sobre un **Coverage Plan aprobado**.
- Misma regla de Review al final.

### 5.3 Review

Toda imagen nueva llega aquí.

**Acciones MVP:**

| Acción | Efecto |
|--------|--------|
| Approve | Asset entra a Library (estado aprobado). |
| Reject | Asset no entra a Library. |
| Edit metadata | Corregir metadata sin regenerar. |
| Regenerate | Nueva generación → vuelve a Waiting Review. |
| Mark duplicate | Marcar como duplicado; no entra como asset útil en Library. |

### 5.4 Library

Solo recursos **aprobados**.

**Sí:** buscar, filtrar, consultar, exportar información.
**No:** generar, revisar, planificar.

### 5.5 Coverage

Responde preguntas accionables:

- ¿Cuántos conceptos existen?
- ¿Cuáles están mal cubiertos?
- ¿Cuáles tienen demasiadas imágenes?
- ¿Cuáles no tienen suficientes representaciones?
- ¿Qué temas faltan?

No es un dashboard decorativo: expone **problemas accionables** (idealmente enlazables a Plans / Factory).

### 5.6 Plans

Define el crecimiento deseado.
Un plan aprobado alimenta Automatic Factory.

### 5.7 Settings

Solo configuración local (rutas, proveedores, preferencias, umbrales).
Sin operaciones de producción.

---

## 6. Propuesta de valor (una frase)

> Visual Library convierte necesidades visuales en una biblioteca conceptual revisada, medible y reutilizable por otras aplicaciones — empezando por VigilCut, sin acoplarse a ella.

---

## 7. Principios de producto (checklist)

- [ ] ¿Esto es un flujo de usuario o una entidad interna? → Si es entidad, no es navegación.
- [ ] ¿Esto salta Review? → Prohibido.
- [ ] ¿Plans está decidiendo el cómo? → Mal.
- [ ] ¿Factory está inventando el qué sin plan/lista? → Mal en Automatic; Manual usa lista explícita.
- [ ] ¿Depende de VigilCut? → Mal.
- [ ] ¿Requiere nube? → Fuera de núcleo MVP.

---

## 8. Métricas de éxito del MVP (producto)

| Métrica | Definición operativa |
|---------|----------------------|
| **Zero-leak to Library** | 0 assets en Library sin Approve. |
| **Reuse rate** | % de necesidades Manual marcadas FOUND vs GENERATE. |
| **Review lag** | Tiempo / cola en Waiting Review. |
| **Coverage actionability** | % de issues de Coverage que pueden convertirse en Plan items. |
| **Plan fidelity** | Automatic Factory solo ejecuta items de planes aprobados. |

---

## 9. Relación con VigilCut (frontera)

| Visual Library | VigilCut |
|----------------|----------|
| Producto dueño del catálogo conceptual | Consumidor |
| Exporta / publica referencias de uso | Importa / referencia assets |
| No conoce pipelines de edición de VigilCut | No escribe en el dominio de VL |

El contrato de integración se diseña **después** del núcleo local (fase posterior al MVP base). En dominio se prevé `AssetUsage` como gancho, sin implementar consumidores.

---

## 10. Glosario de producto (no técnico)

| Término | Definición breve |
|---------|------------------|
| Concepto | Idea visual reutilizable. |
| Representación | Variante de expresión de un concepto. |
| Asset | Archivo + metadata aprobado o en pipeline. |
| Factory | Estación de creación. |
| Review | Estación de control de calidad. |
| Library | Catálogo confiable. |
| Coverage | Diagnóstico de huecos y excesos. |
| Plan | Intención de crecimiento aprobable/ejecutable. |
| FOUND / GENERATE | Decisión de reutilizar vs crear en Manual Factory. |
| Waiting Review | Estado obligatorio post-generación. |

---

## 11. Fuera de este documento

- Modelo formal de dominio → [02-DOMAIN.md](./02-DOMAIN.md)
- Arquitectura técnica → [03-ARCHITECTURE.md](./03-ARCHITECTURE.md)
- Anti-alcance detallado → [13-NON_GOALS.md](./13-NON_GOALS.md)
