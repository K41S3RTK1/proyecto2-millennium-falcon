# Preparación de la entrega

Guion breve y evidencia por criterio: [PRESENTACION.md](PRESENTACION.md).

## Para presentar en vivo

1. Abrir `Iniciar.command` o ejecutar `cargo run --release --offline`. Aparece
   la intro y después la ventana raylib con raytracing GPU. Enter/Espacio salta
   la intro y reproduce TIE antes de la música de ambiente. No hace falta navegador.
2. Probar antes de clase arrastre, flechas y zoom. `Q` alterna Nítido 1200,
   Detalle 1600, Fluido adaptativo y Retina constante. Inicia en Nítido 1200. Usar Fluido
   para recorridos si hace falta; Retina prioriza detalle y puede verse lento. `H` oculta la ayuda.
3. Comprobar que el pie indica **GPU**. `T` alterna con CPU como respaldo.
   Mostrar `1` Principal; luego `2` Motor para enseñar la banda azul y ventiladores.
4. Mostrar `3` Cabina y `4` Refracción. Usar `G` para comparar el vidrio.
5. Usar `F` y `B` para comparar reflexión y skybox; dejarlos activados al terminar.
6. Mostrar `6` Droide, luego `7` Vader y su sable. `C` alterna Tatooine/espacio.
   `I` permite repetir la intro; `M` silencia todo el audio. Al pulsar `7`,
   esperar los 13.653 segundos del sable para escuchar el tema de Vader.
7. Probar `8` Rebeldes: la entrada dispara cuatro láseres (izquierda, derecha,
   izquierda, derecha) con su audio. Pulsar `.` o Disparar
   para lanzar un proyectil con sonido. Cambiar de vista debe detener el efecto.
8. Volver a `1` y pulsar Espacio para el recorrido automático. Espacio lo detiene.
9. Abrir `9` Nave y esperar el ascenso (49.835 s). En el espacio, `E` activa
   el impulso y `F` dispara: tres impactos destruyen cada TIE. Escuchar el sonido
   de cada disparo y explosión junto al ambiente o impulso. `N` repone los TIE;
   `1–8` regresa al diorama. En este modo `F` significa disparar.
10. Tener el video final `renders/diorama-final.mp4` disponible como respaldo.
   Dura 4 min 49 s e incluye audio. El recorrido del diorama empieza en 1:34,
   después de la introducción completa. El modo nave empieza en 3:29.

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
- `renders`: capturas, caras del skybox, video final `diorama-final.mp4` y recorrido inicial `diorama.webm`.
- `Cargo.toml` y `Cargo.lock`: configuración y dependencias.

Los archivos de compilación de `target` se excluyen mediante `.gitignore`.

## Motores GPU y CPU

Incluye el Halcón Milenario, un droide con estación de mantenimiento, Vader,
dos soldados rebeldes y ocho materiales (la rúbrica puntúa hasta cinco), reflexión, refracción, skybox,
cámara orbital y ventana raylib. La versión GPU ejecuta nuestro shader GLSL:
intersecciones, sombras, texturas, reflexión, refracción y cubemap. Rust construye
la escena y su BVH; raylib crea la ventana y ejecuta el shader. La versión CPU
permanece disponible con `T` o `--cpu`. Los modos de nitidez permiten comparar
calidad y rendimiento en Apple M1.

La entrega final reúne la versión GPU, la intro, los dos cielos, Vader, los rebeldes
y los disparos, más el modo nave con combate y sus sonidos. `extras-modo-nave`
conserva el desarrollo de vuelo; `18d74e9` guarda la entrega previa a estos extras.
`escena-cinematica` conserva la rama de desarrollo inicial;
`gpu-raytracing` conserva la migración GPU. Las etiquetas `gpu-estable-2026-09-30`
y `respaldo-antes-rebeldes-b5e459e` permiten recuperar estados anteriores.

El README enlaza el video final y contiene las capturas actuales, incluyendo
los rebeldes, la estela del motor y el combate TIE. Para reproducirlo desde GitHub, abrir
`renders/diorama-final.mp4` o descargarlo con **View raw / Download**.

El docente revisó y autorizó el uso de raylib en esta implementación.
La geometría, materiales y algoritmos de raytracing pertenecen al proyecto.

## Música y video

La intro dura aproximadamente 94.112 segundos: frase azul de 4 segundos y
90.112 segundos del tema musical. Preparar los catorce WAV siguiendo
`assets/audio/README.md`: son archivos locales excluidos de Git, por lo que hay
que copiarlos por separado si se usa otra computadora. Probar los efectos con
`2`, `6`, `7`, `8`, el salto de intro y el cambio de cámaras. Hay una sola pista
principal; en el modo `9`, los disparos y explosiones se mezclan encima del
ambiente o impulso. `M` silencia todo y salir del modo nave detiene sus efectos.

El MP4 final contiene su propia pista de audio; se reproduce sin copiar los WAV.
Se exportó con el renderer del proyecto y se codificó con FFmpeg, sincronizando
los archivos originales. No es una captura de la ventana ni un benchmark:
los 30 cuadros/s del archivo no garantizan ese rendimiento en vivo.

## Verificación de la entrega

Comprobaciones para la actualización del 1 de octubre de 2026:

- `cargo fmt --check`: aprobado.
- `cargo test --offline --locked`: 31 pruebas aprobadas.
- `cargo clippy --offline --locked --all-targets -- -D warnings`: aprobado.
- `--validate-gpu`: 96 comparaciones CPU/GPU aprobadas, incluidos los cuatro momentos de la ráfaga.
- `--validate-flight`: 34 comparaciones CPU/GPU de vuelo y combate aprobadas.
- `--audio-check`: catorce archivos, transiciones, bucles y efectos simultáneos verificados.
- Flechas izquierda/derecha corregidas en ambos modos nativos y en el visor web.
- Los soldados y sus disparos fueron probados en vivo por el autor.

Antes de presentar, probar los controles en la computadora que se utilizará y
tener abierto el MP4 como respaldo. Para demostrar la rúbrica, priorizar rotación
y zoom, cinco materiales, vidrio con `G`, reflexión con `F` y skybox con `B`/`C`.
La intro, Vader, el droide y los rebeldes complementan la presentación visual.

La actualización de calidad y ráfaga incluye suavizado con `J` y Detalle 1600
en las vistas 1–8. El MP4 existente conserva la demostración anterior: no muestra
la nueva ráfaga automática ni el filtro de presentación de la ventana.

## Prueba de sable en rama separada

En `extra-sable-interactivo`, pulsar `7` y luego `L` o el botón Sable para
apagar/encender. Sin sonidos nuevos; conserva su estado al cambiar de vista.
La empuñadura permanece y se apagan hoja, halo y luz roja, también para rayos
secundarios. Se añade `--saber-off` para exportar comparaciones CPU/GPU.
La rama contiene 34 pruebas y la validación GPU amplía la matriz a 128 cuadros.
