# Preparación de la entrega

## Para presentar en vivo

1. Abrir `Iniciar.command` o ejecutar `cargo run --release --offline`. Aparece
   la intro y después la ventana raylib con raytracing GPU. Enter/Espacio salta
   la intro y reproduce TIE antes de la música de ambiente. No hace falta navegador.
2. Probar antes de clase arrastre, flechas y zoom. `Q` alterna Nítido 1200,
   Fluido adaptativo y Retina constante. Inicia en Nítido 1200. Usar Fluido
   para recorridos si hace falta; Retina prioriza detalle y puede verse lento. `H` oculta la ayuda.
3. Comprobar que el pie indica **GPU**. `T` alterna con CPU como respaldo.
   Mostrar `1` Principal; luego `2` Motor para enseñar la banda azul y ventiladores.
4. Mostrar `3` Cabina y `4` Refracción. Usar `G` para comparar el vidrio.
5. Usar `F` y `B` para comparar reflexión y skybox; dejarlos activados al terminar.
6. Mostrar `6` Droide, luego `7` Vader y su sable. `C` alterna Tatooine/espacio.
   `I` permite repetir la intro; `M` silencia todo el audio. Al pulsar `7`,
   esperar los 13.653 segundos del sable para escuchar el tema de Vader.
7. Volver a `1` y pulsar Espacio para el recorrido automático. Espacio lo detiene.
8. Tener el video `renders/diorama.webm` disponible como respaldo. Para grabar otro,
   ejecutar `cargo run --release --offline -- --web` y usar el visor web opcional.

El indicador de imágenes/s mide cuadros nuevos mostrados; no confundirlo con
los FPS de interfaz. En reposo puede marcar cero porque no necesita recalcular.
Si otras aplicaciones saturan la CPU, la resolución adaptativa reduce detalle
durante el movimiento. Mostrar los detalles con la cámara quieta.

## Archivos del proyecto

- `src`: motor de raytracing, escena y visores.
- `tests`: pruebas de intersecciones, óptica, cámara, BVH exportada y renderizado.
- `shaders`: raytracer GPU escrito en GLSL.
- `platform` e `Iniciar.command`: lanzador para macOS.
- `web`: visor opcional para grabar el recorrido.
- `renders`: capturas, caras del skybox y video `diorama.webm`.
- `Cargo.toml` y `Cargo.lock`: configuración y dependencias.

Los archivos de compilación de `target` se excluyen mediante `.gitignore`.

## Motores GPU y CPU

Incluye el Halcón Milenario, un droide con estación de mantenimiento, ocho materiales (la rúbrica puntúa hasta cinco), reflexión, refracción, skybox,
cámara orbital y ventana raylib. La versión GPU ejecuta nuestro shader GLSL:
intersecciones, sombras, texturas, reflexión, refracción y cubemap. Rust construye
la escena y su BVH; raylib crea la ventana y ejecuta el shader. La versión CPU
permanece disponible con `T` o `--cpu`. Los modos de nitidez permiten comparar
calidad y rendimiento en Apple M1.

Antes de entregar, renovar el video de la versión inicial y probar en vivo
rotación, zoom, Q, T, C, I y las siete vistas. La rama `main` conserva la versión CPU
previa; `gpu-raytracing` contiene la migración GPU; `escena-cinematica` añade Vader,
el sable, el segundo cielo y la intro. La etiqueta `gpu-estable-2026-09-30`
permite recuperar la versión anterior a estas mejoras.

El recorrido de la versión inicial está enlazado en el README; las capturas
reflejan la iluminación actualizada. Para reproducirlo desde
GitHub, abrir `renders/diorama.webm` o descargarlo.

## Música y video

La intro dura aproximadamente 94.112 segundos: frase azul de 4 segundos y
90.112 segundos del tema musical. Preparar los siete WAV siguiendo
`assets/audio/README.md`: son archivos locales excluidos de Git, por lo que hay
que copiarlos por separado si se usa otra computadora. Probar los efectos con
`2`, `6`, `7`, el salto de intro y el cambio de cámaras; nunca deben superponerse. El video publicado sigue siendo el recorrido inicial;
falta grabar el recorrido final con las nuevas vistas y la intro antes de entregar.
