# Auditoría reproducible de llamadas — Instant A704x_5in1

> **RECTIFICACIÓN DEFINITIVA (27 oct 2026):** los destinos CALL anómalos de esta auditoría corresponden a un **PE aún transformado por el filtro de compresión x86 de Inno Setup**, no a llamadas del ejecutable instalado. El filtro `InstructionFilter5309` de Inno Setup 5.3.9+ requiere un paso de decodificación después de descomprimir. **La antigua hipótesis de código dinámicamente parcheado no es necesaria**. La firma Authenticode del PE reconstruido proporciona una verificación independiente del resultado. Consulte la sección de corrección al final.


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

## Relocalizaciones

Revisé también el directorio `.reloc` PE: no contiene entradas de reubicación en los cuatro bytes operandos de las cinco instrucciones `CALL rel32` de la tabla. Por tanto, la **reubicación normal de base del cargador PE no explica esos destinos**. Esto no descarta modificaciones especiales del código durante la ejecución ni que se trate de rutas muertas.

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

## Corrección: el filtro Inno explica exactamente los CALL imposibles

La implementación de [innoextract `src/setup/data.cpp`](https://github.com/dscharrer/innoextract/blob/master/src/setup/data.cpp) activa `CallInstructionOptimized` y para Inno Setup 5.3.9+ utiliza `InstructionFilter5309`; [`src/stream/exefilter.hpp`](https://github.com/dscharrer/innoextract/blob/master/src/stream/exefilter.hpp) implementa la inversión de direcciones relativas x86. No son desplazamientos normales de CALL mientras están comprimidos. Los 17 operandos repetidos `E8 BA EA 15 00` representaban el mismo destino almacenado en forma optimizada, aunque sus VAs fueran diferentes.

El PE obtenido inicialmente se cortó en la dirección de la tabla de certificados, `0x251600`. Al recuperar también los `0x2A10` bytes del certificado y aplicar **por archivo** el filtro a partir del byte cero se obtiene un PE firmado correctamente reconstruido.

| Propiedad | Resultado |
|---|---|
| Inicio del PE dentro de la corriente descomprimida | `0x16C81` |
| Longitud del PE reconstruido (certificado incluido) | `0x254010` bytes (2.441.232) |
| SHA-256 del PE restaurado | `4488e47491823de6746564c0a0ec0063ff05328d3d3776200318e45117b1e358` |
| SHA-256 Authenticode calculado | `1445ab08184e1cc7d2ec9342195ed216d51c1f11026fee8f9c61b0e55b0d2591` |
| Digest SHA-256 almacenado en SignedData del PE | **El mismo valor exacto** |
| Instrucciones CALL/JMP transformadas por el decodificador | 49.513 |

La coincidencia del digest autentica **la consistencia de los bytes con la firma incorporada**; no equivale por sí sola a verificar la confianza o vigencia de la cadena de certificados.

### Destinos de CALL después de la restauración

| Dirección de la instrucción | Destino correcto |
|---|---|
| `0x41B54E` | `0x55F6B4` → `HidD_GetFeature` |
| `0x41B593` | `0x55F6BA` → `HidD_SetFeature` |
| `0x41B6A3` | `0x55F6BA` → `HidD_SetFeature` |
| `0x4273F8` | `0x55F6BA` → `HidD_SetFeature` |
| `0x430609` | `0x55F6BA` → `HidD_SetFeature` |

El bloque `0x41B535` realmente llama a **`HidD_GetFeature` de longitud 0x20**, con `buffer[0]=07`. No se ve comprobación inmediata de su retorno en ese fragmento; sí se comprueba `(buffer[3] & 0xFE) >= 0x08`. La variante 1 sale antes de este chequeo, y la variante 0, si lo cumple, llama a `HidD_SetFeature` con longitud `0x08` después de asignar `07 1C 00 20`. Posteriormente se observa una escritura que comienza por `07 20 00 00`. **Sus significados funcionales no están demostrados**, y estas escrituras no se han añadido.

**Reproducción sin ejecutar el PE:** `python tools/restore_inno_5309.py inno_payload.bin -o instant_restored.exe`. El script comprueba la longitud, el SHA-256 y el digest del archivo. La corriente `inno_payload.bin` es el flujo de datos descomprimido obtenido del instalador, no parte del repositorio. Ni el instalador ni el ejecutable propietario se distribuyen.

La tabla inicial de destinos inválidos queda conservada **solo como evidencia del error de extracción**. No debe citarse como comportamiento de ejecución del programa.
