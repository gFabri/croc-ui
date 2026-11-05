# Croc UI

Interfaz de escritorio para [croc](https://github.com/schollz/croc), escrita en Rust con `eframe/egui`.

## Ejecutar

Instala Rust desde <https://rustup.rs/> y asegúrate de que `croc.exe` esté en `PATH`.

```powershell
cargo run --release
```

También puedes escribir en **Configuración avanzada** la ruta completa a `croc.exe`.

## Generar ejecutable Windows

```powershell
cargo build --release
# target\release\croc-ui.exe
```

La interfaz ejecuta `croc send <archivos>` y `croc <código> --out <carpeta>` en un hilo separado, mostrando la salida y errores sin bloquear la ventana.
