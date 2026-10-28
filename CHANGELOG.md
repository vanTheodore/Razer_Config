# Historial de CLON//CONTROL

Este archivo resume los cambios importantes **sin reescribir el historial Git publicado**. Los SHA y PR originales siguen siendo consultables.

## Pendiente de integrar — «El clon responde»

- **DPI primero:** cuatro perfiles directamente accesibles y un único botón de aplicación, sin ritual de simulaciones.
- DPI aislado por pruebas unitarias de RGB, botones y rueda.
- Sección de otras funciones separada y etiquetada como experimental.
- Nuevo sistema de iconos vectoriales de egui que no depende de glifos Unicode; logotipo coherente para ventana y ejecutable Windows.
- README centrado en lo comprobado en el hardware, no en el trabajo forense.
- Fuente: [PR #10](https://github.com/vanTheodore/Razer_Config/pull/10).

## 2026-10-27 — primera función física comprobada

- El propietario confirmó que aplicar el perfil DPI de **200 CPI nominales** cambió perceptiblemente la velocidad del cursor.
- Se registra el resultado con cuidado: no se midió el CPI exacto ni la persistencia del ajuste.
- Evidencia: [comentario PR #8](https://github.com/vanTheodore/Razer_Config/pull/8#issuecomment-6457222374).

## v0.2.6 — ingeniería inversa y GetFeature

- Reversión correcta del filtro `CallInstructionOptimized` de Inno Setup para analizar el software Instant.
- Identificación de llamadas de lectura HID y comandos candidatos, sin habilitar escrituras especulativas.
- [PR #7](https://github.com/vanTheodore/Razer_Config/pull/7), integrada.

## v0.2.5 — diagnóstico y trazas USB

- Comparador offline de capturas USB y procedimientos de investigación reproducibles.
- [PR #6](https://github.com/vanTheodore/Razer_Config/pull/6), integrada.

## v0.2.4 — resultados HID veraces

- Separación entre «Windows aceptó la solicitud» y «el mouse cambió físicamente».
- Registro local del último intento real.
- [PR #5](https://github.com/vanTheodore/Razer_Config/pull/5), integrada.

## v0.2.3 — canal de configuración correcto

- Selección de `MI 1 / FF01:0001 / Feature 07`, con descriptor de ocho bytes, en lugar del mouse HID ordinario.
- [PR #4](https://github.com/vanTheodore/Razer_Config/pull/4), integrada.

## v0.2.2 y v0.2.1 — inspección y DPI

- Diagnóstico completo de las colecciones HID y mejores controles/etiquetas de DPI.
- [PR #3](https://github.com/vanTheodore/Razer_Config/pull/3) · [PR #2](https://github.com/vanTheodore/Razer_Config/pull/2), integradas.

## v0.2 — primera interfaz gráfica

- Herramienta Rust para Windows, interfaz basada en egui, CI MSVC y dependencias actualizadas.
- [PR #1](https://github.com/vanTheodore/Razer_Config/pull/1), integrada.

## Investigación en curso: PR #8

La [PR #8](https://github.com/vanTheodore/Razer_Config/pull/8) está **abierta**, separada de la interfaz estable: retardos configurables, pruebas diagnósticas y posibles funciones experimentales. No se debe confundir su estado con cambios ya integrados.

## Política de historial

- Preferir **una PR por propósito** y títulos claros.
- Usar **squash merge** para iteraciones de la misma función; conservar el contexto en la PR y este changelog.
- **No modificar a posteriori los commits históricos de `main`** ni eliminar ramas con PR abiertas, commits no integrados o análisis pendiente.
- Borrar ramas temporales solo tras comprobar que el PR correspondiente está fusionado y que no contienen trabajo independiente.
