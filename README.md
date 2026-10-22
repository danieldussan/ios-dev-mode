# ios-dev-mode

App de escritorio que te guía para mostrar y activar el **Modo de desarrollador** en un iPhone (iOS 16+), sin Mac ni Xcode. Hecha con Rust, [egui](https://github.com/emilk/egui) e [`idevice`](https://crates.io/crates/idevice).

Idiomas: Español · English · Português (se cambia en vivo desde el pie de la ventana). [Read in English](README.en.md)

<p align="center"><img src="assets/activated.png" alt="Pantalla de modo desarrollador activo" width="320"></p>

## Contenido

- [Qué hace](#qué-hace)
- [Requisitos](#requisitos)
- [Uso](#uso)
- [En Mac](#en-mac)
- [Probado](#probado)
- [Compilar](#compilar)
- [Notas](#notas)
- [Licencia](#licencia)

## Qué hace

1. Detecta el iPhone por USB (modelo, versión de iOS y estado actual del modo desarrollador).
2. Hace aparecer el interruptor **Modo de desarrollador** en *Ajustes > Privacidad y seguridad*.
3. Vigila el teléfono mientras lo activas en Ajustes y el iPhone se reinicia. Sigue el reinicio, la reconexión y el desbloqueo, envía la confirmación y verifica el estado final.

No es 100% automático: el iPhone no se reinicia solo y, tras reiniciar, iOS pide tu código una vez. La app espera y te dice qué hacer en cada paso.

## Requisitos

- iPhone conectado por USB y **de confianza** en este equipo.
- **Windows:** iTunes (apple.com) o *Apple Devices* (Microsoft Store). Aportan el servicio `usbmuxd` en `127.0.0.1:27015`.
- **Linux:** el paquete `usbmuxd`. Se detiene cuando no hay teléfono y vuelve a arrancar al conectar uno.
- Rust 1.85+ solo si compilas desde el código fuente.

## Uso

Descarga el binario de tu sistema desde [Releases](../../releases) o ejecuta desde el código.

- **Windows:** ejecuta `ios-dev-mode-windows-x64.exe`.
- **Linux:** dale permiso de ejecución y ejecútalo (con el servicio `usbmuxd` disponible):

  ```
  chmod +x ios-dev-mode-linux-x64
  ./ios-dev-mode-linux-x64
  ```

Desde el código:

```
cargo run --release
cargo run -- --mock     # probar la interfaz sin teléfono
```

Flujo típico:

1. Conecta el iPhone, desbloquéalo y toca **Confiar**.
2. Pulsa **Activar**. La app muestra la opción en Ajustes.
3. En el iPhone: *Ajustes > Privacidad y seguridad > Modo de desarrollador*, actívalo y toca **Reiniciar** cuando el iPhone te lo pida.
4. Tras el reinicio, desbloquea con tu código y toca **Activar** cuando lo pida.
5. La app lo detecta y muestra *Modo desarrollador activo*. Si no avanza, pulsa **Verificar ahora**.

## En Mac

No hay versión para macOS y no hace falta: Xcode cumple la misma función. Primero instala [Xcode](https://apps.apple.com/app/xcode/id497799835) desde la Mac App Store, ábrelo una vez y acepta la licencia y los componentes adicionales. Luego usa cualquiera de los dos métodos.

### Método 1: automático (solo abrir Xcode)

En algunos Mac basta con tener Xcode instalado y abierto con el iPhone conectado: la opción **Modo de desarrollador** aparece sola en el iPhone.

1. Conecta el iPhone por USB, desbloquéalo y toca **Confiar**.
2. Abre Xcode y espera unos segundos.
3. Revisa el iPhone: *Ajustes > Privacidad y seguridad > Modo de desarrollador*. Si el interruptor ya está, ve al paso 4 del Método 2.

Si el interruptor no aparece, usa el Método 2.

### Método 2: manual

1. Conecta el iPhone por USB, desbloquéalo y toca **Confiar**.
2. En Xcode abre *Window > Devices and Simulators* (`Shift + Cmd + 2`) y espera a que el iPhone aparezca en la lista.
3. En el iPhone abre *Ajustes > Privacidad y seguridad* y busca **Modo de desarrollador**.
4. Actívalo y toca **Reiniciar** cuando lo pida.
5. Tras el reinicio, desbloquea con tu código y toca **Activar** cuando lo pida.

Si el interruptor no aparece, consulta la guía de Apple: [Enabling Developer Mode on a device](https://developer.apple.com/documentation/xcode/enabling-developer-mode-on-a-device).

## Probado

| Dispositivo | iOS | Resultado |
|---|---|---|
| iPhone XR | 18.7.9 | Flujo completo: mostrar opción, activación manual y reinicio, código, confirmación, éxito |
| iPhone 13 | 27.0.1 | Lectura de estado con el teléfono bloqueado, modo ya activo |

El comportamiento en iOS 16/17 no está probado (`reveal` y `enable` están documentados para iOS 18+). Si fallan, la app pasa a instrucciones manuales y sigue verificando el estado.

## Compilar

```
cargo build --release
cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check
cargo run --example spike -- status   # sonda por consola con un teléfono real
```

`idevice` está fijado a `=0.1.68`: trae cambios incompatibles en cada versión menor hasta la 0.2.0. El TLS usa la feature `ring` para que en Windows no haga falta cmake ni NASM.

## Notas

- Windows puede mostrar un aviso de SmartScreen porque el `.exe` no está firmado.
- No está afiliado a Apple. Úsalo en tus propios dispositivos.

## Licencia

[MIT](LICENSE) · hecho por [danieldussan](https://github.com/danieldussan)
