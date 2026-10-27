# CLON//CONTROL 🦀 — Mouse clónico 18F8:1286

Configurador **experimental y no oficial** en Rust para Windows 10/11, diseñado para mouse gaming con VID:PID `18F8:1286` (Instant A704F, como algunas copias del Razer DeathAdder Essential).

**Sin validar en hardware físico.** El build Windows prueba que se puede compilar; no demuestra que la interfaz HID acepte los comandos ni que tengan efecto. No instala un driver, no toca firmware y no reemplaza el HID de Windows.

## Descargar sin compilar

Visita [Releases](https://github.com/vanTheodore/Razer_Config/releases). Después de cada compilación exitosa de `main` se publica una **prerelease experimental** `build-N-SHA` con `ClonControl-Windows-x64.zip` y `SHA256SUMS.txt`.

Las etiquetas Git que empiecen con `v` (por ejemplo `v0.2.0`) publicarán un release de versión **solo si las pruebas y el build terminan correctamente**. Los pull requests compilan sin publicar releases. Las ejecuciones de `workflow_dispatch` generan artefactos, pero no releases.

Los runners públicos estándar de GitHub Actions no consumen tus minutos privados; solo necesitas descargar los binarios al terminar.

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

Se necesita Rust 1.88 o superior y toolchain Windows MSVC.

## Fuente y licencia

Basado en los [reportes documentados del A704F](https://github.com/AmberIsFrozen/Instant-A704F-Mouse-Utilities/blob/main/SPECS.md) por AmberIsFrozen (MIT). El VID/PID por sí solo no garantiza compatibilidad. Código MIT en [LICENSE](LICENSE).

*Razer Synapse no reconoce tu mouse. Nosotros tampoco reconocemos a Razer.* XD
