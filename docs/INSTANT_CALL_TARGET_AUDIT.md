# Auditoría reproducible de llamadas — Instant A704x_5in1

**27 de octubre de 2026.** Evidencia verificada leyendo los bytes del **PE32 original extraído**, sin ejecutarlo. La extracción previa puede necesitar una comprobación independiente antes de inferir comportamiento de ejecución.

## Muestra y método

- SHA-256 del ejecutable interno: `be9d93115074696eaa4d1f382c6b98e0109baea3a53f7482e1adc880a3a865ed`.
- ImageBase: `0x400000`. La sección ejecutable `.text` es `0x401000..0x58CD62`; `.rdata` comienza en `0x58D000` y no tiene bandera IMAGE_SCN_MEM_EXECUTE.
- Para calcular un destino CALL x86 `E8 xx xx xx xx`: `target = call_VA + 5 + sign_extend_i32(rel32)`. El valor `rel32` de la instrucción se extrae del PE, sin inventar un thunk.
- Se coteja con `objdump -p`: la IAT `HidD_GetFeature` está en `0x58D1EC` (thunk `0x55F6B4`) y `HidD_SetFeature` en `0x58D1F0` (thunk `0x55F6BA`).

## Resultados medidos

| Call VA | Operando (E8 + desplazamiento LE) | Destino real | Observación |
|---|---|---|---|
| `0x41B54E` | `E8 B4 EA 15 00` | `0x57A007` | Dentro de otra rutina de tratamiento de cadenas, no en un thunk HID |
| `0x41B593` | `E8 BA EA 15 00` | `0x57A052` | Dentro de la misma rutina, no en un thunk HID |
| `0x41B6A3` | `E8 BA EA 15 00` | `0x57A162` | Primer byte `CC` (**INT3**) |
| `0x4273F8` | `E8 BA EA 15 00` | `0x585EB7` | Dentro de los bytes de otra instrucción |
| `0x430609` | `E8 BA EA 15 00` | `0x58F0C8` | Sección `.rdata`, no ejecutable; datos, no función |

La muestra contiene múltiples ocurrencias del operando repetido `E8 BA EA 15 00`, lo que obliga a considerar código que se transforma, rutas no ejecutadas o problemas de extracción/análisis. **No hay base para elegir una de esas explicaciones sin pruebas adicionales.** No implica por sí mismo que el instalador sea malicioso ni que el hardware sea defectuoso.

Los destinos anteriores no son `0x55F6B4` ni `0x55F6BA`. La presencia de imports en el ejecutable no demuestra que estas llamadas ejecuten `HidD_GetFeature` o `HidD_SetFeature`.

## Consecuencias para la ingeniería inversa

1. **Se revoca la interpretación anterior:** el bloque de `0x41B535` prepara un puntero de buffer, un handle y la longitud `0x20`, pero no se ha demostrado que invoque una lectura HID ni un desbloqueo.
2. Los patrones `07 13 7F`, `07 09`, VID:PID `18F8:1286` y el uso `FF01:0001` siguen siendo observaciones estáticas, no prueba de compatibilidad física.
3. **No añadir comandos nuevos al configurador**, no mandar `07 1C 00 20` ni operar en modos de lectura/escritura especulativos.
4. Comprobar la extracción con una segunda herramienta fiable de Inno Setup y comparar SHA-256 del PE resultante. Después estudiar si se modifica el código durante la carga o si las rutas son inalcanzables.
5. Alternativamente, comprobar en un entorno aislado si el programa original reconoce y controla la unidad. Solo si funciona tendría valor capturar sus transferencias y usar `a704f-trace.exe`.

## Reproducción sin instalar software

```bash
python3 tools/check_instant_pe_calls.py RUTA/AL/EXE_EXTRAIDO.exe
```

El script utiliza solo la biblioteca estándar Python, comprueba el SHA-256 antes de leer direcciones y no ejecuta el binario. No se distribuye el PE propietario.
