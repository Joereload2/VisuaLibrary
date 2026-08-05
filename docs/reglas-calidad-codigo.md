# Reglas de Calidad de Código — Visual Library

**Origen:** `reglas-calidad-codigo.pdf` (instrucciones generales de calidad).  
**Estado:** Adoptadas como norma de trabajo (código **nuevo** y archivos **tocados**).  
**No** se exige reescritura big-bang del legado.

**Instrucción para la IA:** aplica estas reglas automáticamente según el lenguaje o librería que detectes. Si el proyecto ya existe, revisa el código actual contra estas reglas y señala/corrige lo que no las cumpla. No preguntes cuál lista usar: detecta el stack y aplica la sección correspondiente.

**Stack de este repo (prioridad):** Rust · TypeScript/React · SQL/SQLite · CSS.  
Secciones Python / Java / Go: solo si aparece ese stack.

---

## Principios generales (siempre)

### SOLID
Cada clase/módulo con una sola responsabilidad; abierto a extensión, cerrado a modificación; evitar dependencias rígidas entre componentes.

### DRY
No duplicar lógica. Si algo se repite 2+ veces, extraer función/clase.

### KISS
Preferir la solución más simple que resuelva el problema.

### YAGNI
No construir abstracciones o funcionalidad “por si acaso”.

### Nombres descriptivos
Variables, funciones y clases con nombres que expliquen su propósito sin necesidad de comentario adicional.

### Funciones cortas
Una función hace una cosa. Si supera ~30–40 líneas o mezcla responsabilidades, dividir.

### Manejo de errores explícito
Nunca silenciar errores ni “tragarlos” sin log o propagación. Evitar bloques `catch`/`except` vacíos.

### Tests
Si el proyecto tiene suite de tests, todo código nuevo debe incluir cobertura básica (casos normales + al menos un caso límite).

### Comentarios
Solo donde el código no se explica solo (decisiones no obvias, trade-offs, advertencias). No comentar lo evidente.

---

## Rust (crates + Tauri)

- Seguir las [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) oficiales.
- Evitar `unwrap()` / `expect()` sin justificar; usar `Result<T, E>` y propagación con `?`.
  - En **tests** se admite `unwrap`/`expect` si el fallo del test es claro.
- Minimizar `clone()` innecesario; preferir referencias y lifetimes explícitos cuando aporte claridad.
- Aplicar **rustfmt** y respetar advertencias de **clippy**.
- Si usa Tokio/async: evitar bloquear el runtime (nada de I/O síncrono pesado dentro de tareas async sin `spawn_blocking` justificado).
- Si usa Serde: preferir `derive` frente a implementaciones manuales salvo necesidad real.
- Capas del monorepo: `domain` (puro) · `application` (casos de uso + ports) · `infrastructure` (SQLite/FS/HTTP) · no saltarse capas.

---

## JavaScript / TypeScript (packages/ui + shell)

- Preferir TypeScript con tipado estricto sobre JS puro.
- Estilo: base razonable (consistencia del repo); evitar estilo caótico.
- Evitar `any` en TypeScript salvo justificación explícita.
- **React:** componentes pequeños y puros cuando sea viable; hooks personalizados para lógica repetida; evitar prop drilling excesivo (Context o state manager si aplica).
- Promesas con `async/await`; no mezclar con `.then()` en el mismo bloque sin motivo.
- UI: preferir pestañas / paneles acotados frente a scroll de página o superposiciones ilegibles (ver UX del producto).

---

## SQL / Bases de datos (SQLite)

- Normalizar esquema salvo razón explícita para desnormalizar.
- Índices en columnas usadas en `WHERE` / `JOIN` frecuentes.
- Evitar `SELECT *` en código de producción.
- Migraciones versionadas e idempotentes; no editar migraciones ya aplicadas en main sin ADR.

---

## CSS / Frontend visual

- Convención clara de nombres (o componentes con estilos encapsulados).
- Si se usa Tailwind (hoy no es el default del repo): evitar clases repetidas sin extraer a componente.
- Cumplir **WCAG nivel AA** en contraste y accesibilidad en pantallas nuevas o tocadas (labels, alt, teclado).

---

## Python / Java / Go

Aplicar solo si el monorepo incorpora ese stack (ver PDF original / sección genérica del documento fuente). Resumen:

| Lenguaje | Ejes |
|----------|------|
| Python | PEP 8/257, type hints, excepc específicas, capas en FastAPI/Django |
| Java | Effective Java, DI por constructor en Spring, capas |
| Go | gofmt, `if err != nil`, interfaces pequeñas |

---

## Política de aplicación en este repo

| Alcance | Obligación |
|---------|------------|
| **Código nuevo** | Cumplir estas reglas |
| **Archivo tocado** | Dejarlo mejor o igual (no empeorar) |
| **Legado sin tocar** | No reescribir por cumplir el PDF |
| **PR / agente** | `pnpm quality` debe pasar en lo que se toca |

### Comandos

```bash
# Check completo (fmt check, clippy, cargo test, tsc/vitest)
pnpm quality

# Strict: clippy trata warnings como error (opcional / CI futuro)
pnpm quality:strict

# Solo Rust / solo UI
pnpm quality:rust
pnpm quality:ui

# Script PowerShell (Windows)
pnpm quality:ps1
```

Ver también: `scripts/check-quality.ps1`.

---

## Instrucción de uso (copiar a la IA)

```text
Revisa/aplica las reglas de docs/reglas-calidad-codigo.md.
Detecta el stack (Cargo.toml, package.json, …) y aplica la sección
Rust / TypeScript-React / SQL / CSS que corresponda.
Código nuevo y archivos tocados deben cumplirlas.
Si el proyecto no las cumple en lo que tocas, corrige o señala
puntos pendientes. No reescribas el monorepo entero sin pedirlo.
```

---

## Referencias

- Playbook: [`AI_PLAYBOOK.md`](./AI_PLAYBOOK.md)
- Ingeniería: [`constitution/ENGINEERING.md`](./constitution/ENGINEERING.md)
- Testing: [`constitution/TESTING.md`](./constitution/TESTING.md)
- ADR: **D-040** en [`12-DECISIONS.md`](./12-DECISIONS.md)
