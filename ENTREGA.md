# Preparación de la entrega

## Para presentar en vivo

1. Abrir `Iniciar.command` o ejecutar `cargo run --release --offline`. Aparece
   la ventana raylib; no hace falta abrir el navegador.
2. Probar antes de clase arrastre, flechas y zoom. `Q` alterna Nítido 1200,
   Fluido adaptativo y Retina constante. Inicia en Nítido 1200. Usar Fluido
   para recorridos si hace falta; Retina prioriza detalle y puede verse lento. `H` oculta la ayuda.
3. Mostrar `1` Principal; luego `2` Motor para enseñar la banda azul y ventiladores.
4. Mostrar `3` Cabina y `4` Refracción. Usar `G` para comparar el vidrio.
5. Usar `F` y `B` para comparar reflexión y skybox; dejarlos activados al terminar.
6. Mostrar `6` Droide y la estación de mantenimiento.
7. Volver a `1` y pulsar Espacio para el recorrido automático. Espacio lo detiene.
8. Tener el video `renders/diorama.webm` disponible como respaldo. Para grabar otro,
   ejecutar `cargo run --release --offline -- --web` y usar el visor web opcional.

El indicador de imágenes/s mide cuadros nuevos mostrados; no confundirlo con
los FPS de interfaz. En reposo puede marcar cero porque no necesita recalcular.
Si otras aplicaciones saturan la CPU, la resolución adaptativa reduce detalle
durante el movimiento. Mostrar los detalles con la cámara quieta.

## Archivos del proyecto

- `src`: motor de raytracing, escena y visores.
- `tests`: pruebas de intersecciones, óptica, cámara y renderizado.
- `platform` e `Iniciar.command`: lanzador para macOS.
- `web`: visor opcional para grabar el recorrido.
- `renders`: capturas, caras del skybox y video `diorama.webm`.
- `Cargo.toml` y `Cargo.lock`: configuración y dependencias.

Los archivos de compilación de `target` se excluyen mediante `.gitignore`.

## Versión inicial CPU

Incluye el Halcón Milenario, un droide con estación de mantenimiento, cinco materiales, reflexión, refracción, skybox,
cámara orbital y ventana raylib. El trazado se ejecuta en CPU; raylib recibe
los píxeles y presenta la imagen. Los modos de nitidez permiten comparar calidad
y rendimiento en Apple M1.

El recorrido de la versión inicial está enlazado en el README; las capturas
reflejan la iluminación actualizada. Para reproducirlo desde
GitHub, abrir `renders/diorama.webm` o descargarlo.
