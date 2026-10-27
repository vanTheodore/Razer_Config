# CLON//CONTROL 🦀 — Mouse clónico 18F8:1286

Configurador **experimental y no oficial** en Rust para Windows 10/11, diseñado para mouse gaming con VID:PID `18F8:1286` (Instant A704F, como algunas copias del Razer DeathAdder Essential).

**Sin validar en hardware físico.** El build Windows prueba que se puede compilar; no demuestra que la interfaz HID acepte los comandos ni que tengan efecto. No instala un driver, no toca firmware y no reemplaza el HID de Windows.

## Descargar sin compilar

Visita [Releases](https://github.com/vanTheodore/Razer_Config/releases). Después de cada compilación exitosa de `main` se publica una **prerelease experimental** `build-N-SHA` con `ClonControl-Windows-x64.zip` y `SHA256SUMS.txt`.

Las etiquetas Git que empiecen con `v` (por ejemplo `v0.2.0`) publicarán un release de versión **solo si las pruebas y el build terminan correctamente**. Los pull requests compilan sin publicar releases. Las ejecuciones de `workflow_dispatch` generan artefactos, pero no releases.

Los runners públicos estándar de GitHub Actions no consumen tus minutos privados; solo necesitas descargar los binarios al terminar.

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
- MSRV declarada en Cargo.toml: 1.95 (mínimo de egui 0.37).
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
2. Cierra temporalmente reWASD y ejecuta `a704f-diag.exe`. Confirma `18F8:1286` y la colección mouse `0001:0002`.
3. Ejecuta `cloncontrol.exe`. Desmarca DPI, botones y rueda. Activa **Aplicar RGB** y selecciona **Apagada**.
4. Con **Simulación** activada, pulsa **Aplicar**. Deberías ver la secuencia `07 13 7F 17 00 00 00 00` en la vista previa, sin ninguna escritura real.
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
- Toolchain Rust 1.99.1, MSRV 1.95 (MSVC x64 en Windows).

La actualización se verifica con pruebas y build en GitHub Actions. Las dependencias transitivas se resuelven según el registro de Cargo mientras no se versione `Cargo.lock`; las cuatro directas sí están fijadas exactamente. No implica prueba real de hardware.

## Fuente y licencia

Basado en los [reportes documentados del A704F](https://github.com/AmberIsFrozen/Instant-A704F-Mouse-Utilities/blob/main/SPECS.md) por AmberIsFrozen (MIT). El VID/PID por sí solo no garantiza compatibilidad. Código MIT en [LICENSE](LICENSE).

*Razer Synapse no reconoce tu mouse. Nosotros tampoco reconocemos a Razer.* XD
