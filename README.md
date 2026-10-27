# CLON//CONTROL

Configurador experimental en Rust para el mouse HID USB `18F8:1286` (Instant A704F).

> **Sin validar en hardware real.** No es un driver ni software oficial de Razer.

Los builds Windows x64 y prereleases se publicarán automáticamente después de pasar las pruebas. Ver el workflow en `.github/workflows/windows.yml`.

No se modifica firmware ni se instalan drivers. Primera prueba: ejecutar `a704f-diag.exe`, luego simular y ensayar solo el apagado de RGB.

Fuente del protocolo: [Instant A704F Mouse Utilities](https://github.com/AmberIsFrozen/Instant-A704F-Mouse-Utilities) (MIT).

Licencia: MIT.
