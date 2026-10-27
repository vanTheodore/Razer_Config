# Instant A704x_5in1 — análisis estático dirigido al protocolo HID

> **Corrección del análisis, 27-oct-2026:** las direcciones de llamada reales del PE extraído no permiten atribuir el bloque de 32 bytes a las funciones HID. Se encontraron llamadas cuyo destino está dentro de otra función, en una instrucción INT3 o en datos. Las secciones 4 y 6 de este documento describen patrones de bytes, **no una negociación demostrada**. Véase [auditoría de destinos de llamada](INSTANT_CALL_TARGET_AUDIT.md).


**27 de octubre de 2026.** Investigación del instalador aportado por el propietario del mouse. **Ni el instalador ni el ejecutable se han ejecutado.** No se agregan binarios propietarios ni comandos USB nuevos.

## Artefactos

- Instalador Inno Setup 5.5.7: SHA-256 `e677b33e955d5c5cbb14ef6120b45a4bf5af8d498b911f34920b0dfc10cd417c`.
- Ejecutable interno PE32 x86 (2.430.464 bytes): SHA-256 `be9d93115074696eaa4d1f382c6b98e0109baea3a53f7482e1adc880a3a865ed`.
- Importaciones observadas: `HidD_SetFeature`, `HidD_GetFeature`, `HidD_GetAttributes`, `HidP_GetCaps`, APIs SetupAPI y `CreateFileW`.
- Las direcciones indicadas son **VA preferidas del ejecutable interno**, no offsets del instalador; documentan patrones verificables de instrucciones, no ejecución demostrada.

## 1. Soporte específico del identificador USB

Constantes de 32 bits en `.data` y comparaciones en `0x414789–0x414846`:

| VID | PID | Variante interna | Otro campo interno |
|---|---|---|---|
| **`18F8`** | **`1286`** | **0** | `0x0C` |
| `30FA` | `1701` | 1 | `0x13` |
| `18F8` | `0F99` | 2 | `0x05` |

Constantes en `0x5E6014..0x5E6028`. Este ejecutable **sí contempla literalmente nuestro VID:PID**. Los valores del último campo no han sido interpretados como opcodes.

## 2. Canal HID de fabricante

En `0x414954–0x4149BA` se comprueba `UsagePage == 0xFF01` y `Usage == 1`. Se prepara un handle con acceso `0x40000000` (`GENERIC_WRITE` en Windows), que se almacena en `0x5ECB8C`. Coincide con la colección `MI 1 / FF01:0001` y Feature Report 07 de ocho bytes observada en el descriptor del mouse del usuario.

## 3. Construcción de reportes

Se observa un buffer global que comienza en `0x5EC848`:

| Secuencia construida | Direcciones de evidencia | Inferencia |
|---|---|---|
| `07 13 7F XX` | `0x4273D0–0x4273F8`, `0x4305E1–0x430609` | Modo de iluminación |
| `07 09 XX YY` | `0x4274AA–0x4274B9`, `0x43069C–0x4306AB` | Configuración DPI |
| `07 12` | `0x425C8D–0x425D08` | Otra opción tipo frecuencia |
| `07 15`, `07 16` | `0x411C20–0x411DC7` | Opciones de entrada adicionales |
| `07 20 00 00` (prefijo) | `0x4185E2` | Semántica **no establecida** |

El cálculo de `XX` en la rutina RGB para índices de estado 0–3 corresponde a `0x10, 0x13, 0x16, 0x17`. En particular, el `0x17` coincide con el valor que CLON//CONTROL ya usaba para apagar el RGB. **No está justificado sustituirlo por valores inventados**.

La longitud de los reportes procede de un campo de 16 bits en `0x5EC808`. El programa copia allí una estructura compatible con `HIDP_CAPS` (en `0x414968–0x414976`); el campo a desplazamiento 8 de `HIDP_CAPS` es `FeatureReportByteLength`. Esto sugiere que el programa utiliza la longitud anunciada por HID y no simplemente el literal 8 en todos los casos.

## 4. Intercambio previo posible (NO IMPLEMENTAR todavía)

Cerca de `0x41B535–0x41B5A7`, antes de las configuraciones, el software prepara un buffer cuyo primer byte es `07`, una operación con longitud aparente de **32 bytes** y examina posteriormente datos del buffer. En una rama condicional de la variante 0 se observan escrituras compatibles con un encabezado `07 1C 00 20` y una operación posterior con longitud **8 bytes**.

**Incertidumbres:** falta demostrar qué funciones invoca realmente este bloque, el sentido de las transferencias, si los bytes proceden de una respuesta y qué condiciones disparan la rama. La operación de 32 bytes tampoco coincide directamente con los 8 bytes declarados por el Feature Report 07 en Windows. No podemos llamar a esto «desbloqueo» o «inicialización necesaria» con certeza, ni sería responsable enviar esos bytes al mouse a ciegas.

## 5. Interpretación del fracaso anterior

La prueba registrada en CLON//CONTROL envió `07 13 7F 17 00 00 00 00` a `MI 1/FF01:0001` y HIDAPI devolvió éxito, **pero la luz no cambió**. La similitud del encabezado RGB con el programa del fabricante indica que falta comprobar **estado/negociación, longitud, otra condición del firmware o compatibilidad real** antes de alterar el encoder.

## 6. Próximos pasos

1. Desentrañar el destino real de las llamadas indirectas del bloque `0x41B535` y el tratamiento de su respuesta; no confiar únicamente en las importaciones PE.
2. Precisar si `HidD_GetFeature` se llama con 32 bytes y bajo qué precondiciones se envía el eventual `07 1C 00 20`.
3. Mantener CLON//CONTROL sin una secuencia de inicio nueva hasta poder verificarla.
4. Si se comprueba en un entorno controlado que el software original funciona en esta unidad, utilizar `a704f-trace.exe` para comparar exclusivamente las solicitudes del mouse.

**Estado: investigación estática reproducible, sin demostración todavía de una solución funcional del RGB o DPI.**
