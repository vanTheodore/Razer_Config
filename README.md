# CLON//CONTROL 🦀 — Mouse clónico 18F8:1286

Configurador **experimental y no oficial** en Rust para Windows 10/11, diseñado para mouse gaming con VID:PID `18F8:1286` (Instant A704F, como algunas copias del Razer DeathAdder Essential).

**Sin validar en hardware físico.** El build Windows prueba que se puede compilar; no demuestra que la interfaz HID acepte los comandos ni que tengan efecto. No instala un driver, no toca firmware y no reemplaza el HID de Windows.

## Descargar sin compilar

Visita [Releases](https://github.com/vanTheodore/Razer_Config/releases). Después de cada compilación exitosa de `main` se publica una **prerelease experimental** `build-N-SHA` con `ClonControl-Windows-x64.zip` y `SHA256SUMS.txt`.

Las etiquetas Git que empiecen con `v` (por ejemplo `v0.2.0`) publicarán un release de versión **solo si las pruebas y el build terminan correctamente**. Los pull requests compilan sin publicar releases. Las ejecuciones de `workflow_dispatch` generan artefactos, pero no releases.

Los runners públicos estándar de GitHub Actions no consumen tus minutos privados; solo necesitas descargar los binarios al terminar.

## CLON//CONTROL 0.2.3 — canal HID correcto para 18F8:1286

**Diagnóstico aportado por el propietario (Windows):** seis colecciones HID, y solo `MI 1 / Usage FF01:0001` anuncia `Feature Report ID 07`. Su descriptor `85 07 ... 75 08 95 07 B1 02` declara **siete bytes de datos más el ID 07**, por tanto el buffer HIDAPI debe tener **8 bytes**. La colección del mouse `MI 0 / 0001:0002` solo declara entrada y no es un canal de configuración.

En esta versión la app **solo selecciona FF01:0001 en MI 1** si comprueba el descriptor y sus 56 bits para Feature 07. Antes de enviar cada reporte vuelve a comprobar VID:PID, MI, Usage y descriptor: si ya no coincide, no escribe nada. La alternativa experimental de 16 bytes se retira porque no coincide con el descriptor de Windows observado. Se mantiene simulación y confirmación por operación.

**Prueba mínima:** Iluminación → «Preparar prueba: apagar la luz (solo RGB)» → Simular → desactivar simulación → autorizar → Aplicar. Si Windows acepta el envío, observar si se apaga la luz. **No cambies DPI ni botones hasta verificar la prueba.** Un Feature Report declarado y una llamada aceptada no garantizan que el firmware responda.

## CLON//CONTROL 0.2.2 — diagnóstico completo sin escrituras

Tras el diagnóstico real de un mouse identificado como `INSTANT 18F8:1286`, se observó que el descriptor de 66 bytes de la colección MI 0 / Usage 0001:0002 no anunciaba Feature Reports. Sin embargo, también hay colecciones de MI 1: teclado, consumidor y dos usos privados FF00 y FF01.

La utilidad `a704f-diag.exe` ahora intenta leer los descriptores de **todas las colecciones HID**. Muestra IDs Input/Output/Feature, longitud, uso, interfaz, posibles errores de permisos y bytes hexadecimales de cada descriptor. No abre interfaces para escribir ni ejecuta HID SetFeature. El informe local se sobrescribe en `%LOCALAPPDATA%\\ClonControl\\diagnostico.txt`; no incluye rutas físicas USB ni números de serie. De momento no probamos comandos HID adicionales sin esos resultados.

Este diagnóstico no confirma por sí solo que el mouse acepte configuraciones; sí discrimina si MI 1 declara reportes de configuración.

## CLON//CONTROL 0.2.1 — DPI, iluminación y diagnóstico

**Compatibilidad pendiente:** se eligió 18F8:1286 porque la investigación publicada lo asocia al chip A704F, pero el VID/PID no prueba que el firmware de tu unidad acepte sus comandos.

Para cambiar el DPI **de verdad**: Sensibilidad → elegir perfil activo («Usar») → ajustar sensibilidad → Simular → desmarcar «Solo simular» → marcar autorización → «APLICAR AL MOUSE». La pantalla muestra el valor **previsto**, no una lectura del DPI físico. Se envían los cuatro perfiles si se activa «Incluir DPI».

Para probar iluminación: Iluminación → «Preparar prueba: apagar la luz» → Simular → desmarcar simulación, autorizar y aplicar. Si no cambia nada, ejecuta `a704f-diag.exe`: genera un informe de **solo lectura** en `%LOCALAPPDATA%\\ClonControl\\diagnostico.txt`, sin número de serie ni ruta USB. Comprueba el canal FF01:0001 / Feature 07 en Diagnóstico antes de enviar.

**Importante:** la investigación de Linux muestra `07 13 7F 17 00 00 00 00` como comando de apagar la luz; su aplicación original envía por HIDAPI una trama de 16 bytes que comienza por `21 09 07 03 01 00 08 00`. En Windows, `HidD_SetFeature` trata el **primer byte como Report ID**; no sabemos todavía si ninguna de las dos tramas es admitida por tu dispositivo. **Nota histórica:** la trama de 16 bytes ya no está disponible en la aplicación; el descriptor Windows de este mouse anuncia un buffer de ocho bytes para ID 07.

Un resultado «enviado» solo significa que la llamada HIDAPI devolvió éxito; **no confirma que haya cambiado el mouse**.

## CLON//CONTROL 0.2 — interfaz fácil de usar

Interfaz original de tema oscuro inspirada en el flujo de Logitech G HUB, pero sin su marca ni componentes visuales propietarios:

- **Inicio:** resumen de dispositivo, DPI e iluminación, con acceso directo a una primera prueba segura.
- **Sensibilidad:** cuatro controles de DPI, velocidad activa y perfiles habilitados.
- **Iluminación:** modos documentados, incluido apagado (no se promete RGB por color).
- **Botones:** reasignación de botones laterales, sin tocar clic izquierdo y derecho.
- **Rueda:** desplazamiento normal o control de volumen.
- **Diagnóstico:** interfaces HID detectadas y vista previa hexadecimal.
- **Barra inferior fija:** simulación, autorización, aplicación de cambios y guardado del preset.

Los controles modificados se marcan para su próxima aplicación. **Cambiar un valor nunca escribe en el mouse por sí solo**; la confirmación se restablece tras cada modificación o envío.

## Rust y dependencias actualizadas

- Rust **1.99.1** fijado en `rust-toolchain.toml` (estándar estable publicado en octubre de 2026).
- MSRV declarada en Cargo.toml: 1.99 (el objetivo es compilar con Rust 1.99.1).
- eframe/egui **0.37.0**, hidapi **2.6.7**, serde **1.0.229**, serde_json **1.0.154**.
- GitHub Actions actualizadas para compilar, ejecutar pruebas y publicar una prerelease experimental después de un build exitoso.
- El build adjunta `Cargo.lock` para registrar la resolución exacta de dependencias de la versión publicada. En este repositorio aún no se versiona ese lockfile.

## Funcionalidades

- Aplicación gráfica Windows con eframe/egui.
- Diagnóstico HID separado (`a704f-diag.exe`) **sin escrituras USB**.
- Detecta exclusivamente `18F8:1286` (ignora el dispositivo virtual `25B1:0003` de reWASD).
- Cuatro perfiles DPI nominales (200–3000 CPI), modos de iluminación, rueda y dos botones laterales.
- Previsualización de reportes HID y modo simulación por defecto.
- Exige desactivar la simulación y autorizar expresamente cada sesión de escritura.
- Presets locales en `%LOCALAPPDATA%\\ClonControl\\a704f.json`; no lee configuración del firmware.

## Primera prueba

1. Descarga y descomprime el ZIP del release en Windows.
2. Cierra temporalmente reWASD y ejecuta `a704f-diag.exe`. Confirma `18F8:1286` y la colección de configuración `MI 1 / FF01:0001`, Feature 07.
3. Ejecuta `cloncontrol.exe`. Desmarca DPI, botones y rueda. Activa **Aplicar RGB** y selecciona **Apagada**.
4. Con **Simulación** activada, pulsa **Simular**. Deberías ver la secuencia `07 13 7F 17 00 00 00 00` en la vista previa, sin ninguna escritura real.
5. Solo tras identificar la interfaz: desactiva **Simulación**, marca la autorización y aplica. Comprueba la luz del mouse. Si falla, copia el mensaje o el diagnóstico.

Los cambios pueden perderse al desconectar el mouse. No se ejecutan escrituras al inicio de la aplicación.

## Compilar localmente (opcional)

```powershell
cargo test --all-targets
cargo build --release --bins
```

Se necesita Rust 1.99.1 (fijado en rust-toolchain.toml) y toolchain Windows MSVC.


## Dependencias / Rust

Versiones directas fijadas para la serie experimental 0.2 (octubre de 2026):
- eframe/egui 0.37.0 (migrado a `eframe::App::ui`).
- hidapi 2.6.7 (`windows-native`).
- serde 1.0.229.
- serde_json 1.0.154.
- Toolchain Rust 1.99.1, MSRV 1.99 (MSVC x64 en Windows).

La actualización se verifica con pruebas y build en GitHub Actions. Las dependencias transitivas se resuelven según el registro de Cargo mientras no se versione `Cargo.lock`; las cuatro directas sí están fijadas exactamente. No implica prueba real de hardware.

## Fuente y licencia

Basado en los [reportes documentados del A704F](https://github.com/AmberIsFrozen/Instant-A704F-Mouse-Utilities/blob/main/SPECS.md) por AmberIsFrozen (MIT). El VID/PID por sí solo no garantiza compatibilidad. Código MIT en [LICENSE](LICENSE).

*Razer Synapse no reconoce tu mouse. Nosotros tampoco reconocemos a Razer.* XD
