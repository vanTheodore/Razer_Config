# Instant A704x: investigación reproducible (CLON//CONTROL 0.2.5)

## Evidencia extraída del instalador del usuario

- Archivo original: `04d85fe126f275f4f4e74d2967c0ce52.exe`, Inno Setup 5.5.7 Unicode; SHA-256 `e677b33e955d5c5cbb14ef6120b45a4bf5af8d498b911f34920b0dfc10cd417c`.
- Se extrajo sin ejecutarlo un PE32 x86 con SHA-256 `be9d93115074696eaa4d1f382c6b98e0109baea3a53f7482e1adc880a3a865ed`.
- Recurso/nombre: `英斯特A704x_5in1游戏鼠标`.
- Cadenas verificadas: `LED_STATUS`, `DPI_SPEED`, `DPI_LEVEL`, `WHEEL_STATUS`, `KEYMAP` y otras preferencias.
- Importa `HidD_SetFeature`, `HidD_GetFeature`, `HidD_GetAttributes`, `HidP_GetCaps` y varias APIs de SETUPAPI.
- **No hemos reconstruido todavía la secuencia de `HidD_SetFeature`** del controlador. Las cadenas de ajustes no son paquetes HID.
- No hay prueba de que el ejecutable controle la unidad `INSTANT 18F8:1286` del usuario. Nunca se ejecutó el instalador durante esta investigación.

No se incluye ningún binario propietario ni se agrega una dependencia del instalador a los releases.

## Lo que hace realmente la implementación nueva

`a704f-trace.exe` es un **analizador offline**, escrito en Rust. Lee dos exportaciones TSV de TShark: una de referencia y otra obtenida al hacer un cambio concreto en la aplicación original. Enumera los **SET_REPORT** capturados (HID tipo Output/Feature, `bRequest=9`), identifica `wValue`, interfaz (`wIndex`), report ID y payload, y muestra cuáles aparecen con mayor frecuencia en la segunda captura. Conserva todas las tramas sin corregirlas o deducir bytes ausentes. **Nunca escribe al USB.**

Si la aplicación original no consigue cambiar la luz, la comparación tampoco demuestra compatibilidad. Una coincidencia en capturas es solamente una hipótesis hasta probarla en el hardware correcto.

## Capturas de prueba (opcionales; no instales nada a ciegas)

**Primero confirma que el software Instant realmente reconoce el mouse y funciona.** Si no lo reconoce o no cambia ninguna opción, no tiene sentido grabar USB todavía.

Si decides capturar tráfico de tu propio mouse con Wireshark + USBPcap, aísla únicamente el dispositivo correcto, no grabes teclado ni otros USB y evita compartir un PCAP completo. Obtén dos capturas comparables, una de referencia sin cambios y otra cambiando **solo una** opción, por ejemplo la iluminación. Filtra SET_REPORT y exporta los campos mediante TShark:

```powershell
tshark -r ".\\sin_cambio.pcapng" -Y "usb.setup.bRequest == 0x09" -T fields -e frame.number -e usb.setup.bRequest -e usb.setup.wValue -e usb.setup.wIndex -e usb.capdata -e usb.data_fragment > sin_cambio.tsv
tshark -r ".\\luz_apagada.pcapng" -Y "usb.setup.bRequest == 0x09" -T fields -e frame.number -e usb.setup.bRequest -e usb.setup.wValue -e usb.setup.wIndex -e usb.capdata -e usb.data_fragment > luz_apagada.tsv
a704f-trace.exe sin_cambio.tsv luz_apagada.tsv
```

El TSV admite exactamente seis columnas, separadas por tabulaciones, en el orden que se ve arriba. Un valor hex vacío significa que TShark no capturó el payload de esa transferencia: no se reconstruye automáticamente. Algunas versiones de Wireshark exponen `usb.capdata` y otras `usb.data_fragment`; se prefiere el último si está presente.

**Privacidad:** confirma visualmente que los TSV solo contienen transferencias hacia el mouse. No adjuntes PCAPs completos sin revisar; pueden incluir comunicaciones ajenas al mouse.

## Siguiente etapa, todavía bloqueada por evidencia

Comparar el comando que efectivamente envíe el software original con el que hoy enviamos (`07 13 7F 17 00 00 00 00`). Solo después de disponer de una captura real del mouse que demuestre que el software original funciona se debería implementar otro encoder HID. No activar macros, DPI ni firmware desconocidos en modo exploratorio.

## Límites del programa actual

La interfaz principal de CLON//CONTROL conserva su protocolo HID experimental heredado. **No se ha demostrado que funcione en la unidad del usuario**. No se modifica aquí ni se afirma compatibilidad oficial.
