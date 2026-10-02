# Millennium Falcon — Docking Bay 94

Diorama de Star Wars construido con **5,902 bloques** y renderizado por raytracing
implementado desde cero en Rust y GLSL. La nave está estacionada en un pequeño puerto
espacial desértico, con edificios de arenisca, luces, carga, un depósito de cristal,
un droide astromecánico junto a su estación de mantenimiento, Vader con sable rojo
y dos soldados rebeldes junto a la rampa.

[![Millennium Falcon en el puerto espacial](renders/falcon.png)](renders/diorama-final.mp4)

**[▶ Ver el video final con audio](renders/diorama-final.mp4)** — 4 min 49 s,
1200×800, 30 cuadros/s, MP4 H.264 y audio AAC estéreo. Incluye la introducción
completa, giro de 360° y zoom, materiales, comparaciones de reflexión y refracción,
el droide, Vader, los rebeldes con disparos, el skybox espacial y el modo nave
con ascenso, impulso y combate contra tres TIE con audio.

El video se exportó cuadro a cuadro con el renderer del proyecto y se codificó
con FFmpeg, sincronizando los WAV locales con las escenas. Es un recorrido
renderizado: sus 30 cuadros/s no representan el rendimiento en vivo. FFmpeg es
una herramienta de producción del video, no una dependencia de la aplicación.
Si GitHub no muestra el reproductor, descargar el MP4 con **View raw / Download**.

| Tiempo | Demostración |
|---|---|
| 0:00 | Introducción completa con música |
| 1:34 | Rotación de 360° y acercamiento/alejamiento |
| 1:54 | Motor y materiales |
| 2:03 | Cabina: reflexión activada/desactivada |
| 2:17 | Depósito: refracción activada/desactivada |
| 2:30 | Droide y estación de mantenimiento |
| 2:36 | Vader, sable y música de su vista |
| 2:55 | Rebeldes y tres disparos alternados |
| 3:07 | Entorno espacial y comparación del skybox |
| 3:29 | Modo nave: ascenso completo y giro a ambos lados |
| 4:19 | Llegada al espacio y tres cazas TIE |
| 4:23 | Impulso, nueve disparos y tres explosiones con audio |
| 4:43 | Victoria y regreso del ambiente espacial |

El [recorrido inicial sin las últimas mejoras](renders/diorama.webm) se conserva
como referencia. Para la presentación en vivo, consultar [ENTREGA.md](ENTREGA.md).

## Modo nave y combate espacial

Integrado en `main` desde la rama `extras-modo-nave`. El estado previo al modo
nave se conserva en el commit `18d74e9`. El video final incluye ambos modos.

**9 / Nave** abre una escena independiente con sólo el Halcón a escala 1.65,
con rampa y tren recogidos. Asciende durante los **49.835 s** de `Despegue.wav`:
la cámara acompaña la nave y la atmósfera se oscurece hasta convertirse en espacio.
Al terminar comienza `ambientefalcon.wav` en bucle (**97.621 s** por vuelta).

En el espacio, **E** o **Impulso** inicia una estela celeste desde el arco del motor
trasero y reproduce `Impulso.wav` completo (**19.797 s**). Pulsar otra vez reinicia
el impulso. Al terminar se desvanece la estela y vuelve la música espacial.
Durante el ascenso el impulso permanece deshabilitado. Hay una sola pista
principal, acompañada por efectos de combate; **M** conserva su estado al entrar y salir de este modo.

![Halcón en modo nave con la estela activa](renders/modo-nave.png)

Al llegar al espacio aparecen **tres cazas TIE** de bloques, con cabina y paneles
plegados inspirados en la referencia. **F** dispara automáticamente al primer
objetivo vivo. Hay un proyectil en vuelo a la vez (0.55 s); cada caza requiere
**tres impactos**, muestra un destello al recibir daño y explota al tercero.
El contador muestra la vida restante; al destruir los tres aparece la victoria.
**N** repone la oleada sin repetir el ascenso. **E** funciona durante el combate;
cada disparo aceptado reproduce `disparofalcon.wav` y cada TIE destruido reproduce
`explotion.wav`, sobre el ambiente o impulso. Los efectos consecutivos pueden
superponerse sin recortarse. **M** silencia todo; **N**, **9** y salir del modo
nave detienen los efectos pendientes.

![Combate contra tres cazas TIE](renders/combate-tie.png)

- **F**: disparar; **N**: nueva oleada (sólo en el espacio).
- **1–8**: regresar a la vista correspondiente del diorama.
- **9**: repetir el ascenso completo.
- **Arrastre/flechas y rueda/+/-**: girar y acercar/alejar la cámara.
- **R**: restablecer la cámara de vuelo.
- **Q**: alternar 1200 → 1600 → 800 píxeles de ancho; **T**: alternar GPU y respaldo CPU.
- **J**: activar/desactivar suavizado de bordes en GPU.
- **S**: guardar `renders/vuelo-captura.png`.

La escena de vuelo tiene su propia geometría y BVH. Los TIE se agregan al llegar
al espacio y se retiran al destruirse; el BVH sólo se reconstruye en esos cambios. La traslación se aplica a los
rayos sin reconstruir los bloques por cuadro. La estela es una lámina emisiva
integrada a lo largo del rayo, recortada por el primer impacto; CPU y GPU calculan
el mismo efecto. No pretende simular dinámica de fluidos. Los modos gráficos y
las dimensiones del diorama original se conservan al regresar.

Los cinco WAV del modo nave se colocan en `assets/audio/` y siguen excluidos de Git.
Se pueden verificar imágenes y audio sin interactuar con la ventana:

```sh
cargo run --release --offline -- --flight-preview --flight-progress 1 --boost-age 1 --output renders/vuelo-captura.png
cargo run --release --offline -- --combat-preview --combat-shot-age .3 --output renders/vuelo-captura.png
cargo run --release --offline -- --validate-flight
cargo run --release --offline -- --audio-check
```

La verificación incluye 34 pruebas, 34 comparaciones CPU/GPU del vuelo y combate,
las 128 del diorama y los catorce audios. Las flechas horizontales usan el mismo
sentido en el diorama, el modo nave y el visor web: izquierda gira la escena a
la izquierda y derecha a la derecha.

Para reproducir el segmento de vuelo y añadirlo al video base anterior (requiere
FFmpeg y ffprobe, únicamente para producir el video):

```sh
cargo run --release --offline --example export_flight_video -- /tmp/vuelo-silent.mp4
python3 scripts/compose_delivery_video.py /ruta/diorama-base.mp4 /tmp/vuelo-silent.mp4 /tmp/diorama-final.mp4
```

El video base corresponde a `renders/diorama-final.mp4` del commit `18d74e9`.

## Ejecutar

Requiere Rust y Cargo con soporte para la edición 2024. En macOS también se
necesitan las herramientas de compilación de Xcode y CMake para compilar raylib.

**En Mac: doble clic en `Iniciar.command`.** Compila en modo optimizado y abre una
ventana independiente. Mantener el lanzador dentro de esta carpeta. También se
puede ejecutar desde Terminal, dentro del proyecto:

```sh
cargo run --release --locked
```

Después de descargar las dependencias una vez se puede agregar `--offline`.
Usar siempre `--release` para presentar; la compilación de depuración es más lenta.
El lanzador genera una app local en `target/Millennium Falcon.app`.
`cargo run --release --offline -- --demo` activa el recorrido después del prólogo.
Al abrir siempre aparece la introducción: 4 segundos de frase azul y 90.112
segundos de música y títulos. Enter o Espacio permiten saltarla.
Después, la ventana inicia con el **raytracer GPU**; `--cpu` permite usar el motor CPU de
respaldo. Si el shader no compila, la aplicación vuelve automáticamente a CPU.

### Controles de la ventana

| Acción | Control |
|---|---|
| Rotar alrededor del diorama | Arrastrar sobre la escena o usar las flechas |
| Acercar y alejar | Rueda/scroll del trackpad, caracteres `+` y `−` (también `=`), teclado numérico o botones `+` / `−` |
| Vista inicial | `R` o `1` |
| Motor / Cabina / Refracción / Superior / Droide | `2` / `3` / `4` / `5` / `6`, o botones superiores |
| Recorrido automático / detener | Espacio |
| Activar/desactivar reflexión | `F` |
| Activar/desactivar refracción | `G` |
| Activar/desactivar skybox | `B` |
| Nitidez durante el movimiento | `Q` o botón superior: Nítido 1200 → Detalle 1600 → Fluido → Retina |
| Suavizado de bordes en GPU | `J` (también disponible en modo nave) |
| Alternar raytracer CPU / GPU | `T` (misma cámara y efectos) |
| Vader y sable | `7` o botón Vader |
| Encender/apagar sable (sólo en la vista 7) | `L` o botón Sable; sin audio adicional |
| Soldados rebeldes | `8` o botón Rebeldes |
| Modo nave y combate | `9` o botón Nave |
| Impulso, únicamente en el espacio del modo nave | `E` o botón Impulso |
| Disparo manual en la vista de rebeldes | `.` / decimal numérico o botón Disparar |
| Cambiar Tatooine / espacio | `C` |
| Repetir introducción | `I` |
| Saltar introducción | Enter o Espacio |
| Silenciar / activar todo el audio | `M` |
| Mostrar/ocultar ayuda | `H` |
| Guardar imagen de calidad | `S` → `renders/captura.png`, 2560 px de ancho (mantener cámara quieta) |
| Salir | Escape o cerrar la ventana |

### Rendimiento en Mac M1

La mejora desarrollada en `mejora-imagen-fluida` e integrada en `main` añade un pase de suavizado de bordes
en GPU, activado inicialmente y comparable con `J`. En vuelo inicia a 1200 px;
`Q` permite probar 1600 para más detalle u 800 para priorizar fluidez. El diorama mantiene sus modos de
resolución y refinamiento anteriores. El filtro afecta sólo a la presentación de
la ventana: las capturas `S`, los renders exportados y el video de entrega
conservan la salida del raytracer. Ver [mediciones y límites](MEJORA_IMAGEN.md).

El modo GPU ejecuta un fragment shader GLSL 330 mediante raylib/OpenGL 4.1.
Cada píxel calcula sus propios rayos en paralelo. La geometría sigue siendo una
colección de bloques: el shader implementa intersecciones, texturas, iluminación,
sombras, reflexión, refracción y skybox. No utiliza un motor de raytracing externo
ni depende de aceleración de rayos dedicada del hardware.

- La escena y la BVH se cargan una sola vez en texturas RGBA32F. El recorrido
  visita primero las cajas más cercanas; las sombras terminan al encontrar un
  obstáculo opaco. La pila BVH admite 64 entradas; la construcción limita la profundidad a 60.
- La imagen permanece en GPU para presentarla; solo las capturas y las pruebas
  comparativas la leen de vuelta a CPU.
- Inicia en **Nítido 1200**, con ancho fijo durante el movimiento. `Q` cambia a
  **Detalle 1600** (ancho fijo, disponible en las ocho vistas),
  **Fluido adaptativo** (480–1200 px en GPU), **Retina constante** (1600–2560 px)
  y regresa a Nítido. El modo Fluido busca 30 imágenes/s, sin garantizarlas.
- Todos los modos conservan los efectos. En movimiento usan una muestra por
  píxel y tres niveles de rayos secundarios. Al detenerse recuperan oclusión
  local, seis niveles y finalmente cuatro muestras por píxel a resolución Retina.
- La ventana tiene un límite de 60 FPS. Las vistas cercanas del vidrio exigen más
  rayos; para priorizar fluidez en esas vistas se puede seleccionar Fluido.
- `T` cambia a CPU sin perder encuadre ni controles. El motor CPU conserva el
  reparto por bandas de ocho filas, cancelación del refinamiento y resolución
  adaptativa de 192–640 px. No calcula imágenes en segundo plano mientras se usa GPU.
- **Imágenes/s** cuenta actualizaciones presentadas; **UI FPS** mide el ciclo de
  la ventana. El tiempo en ms corresponde al trazado en CPU y al intervalo del
  ciclo de la ventana en GPU. En reposo no se recalcula continuamente.

Para medir cuadros GPU terminados, el benchmark fuerza lectura de la textura:

```sh
cargo run --release --offline -- --benchmark-gpu --width 1200 --height 644 --quality 0
```

El resultado incluye transferencia y copia de píxeles; no mide solamente el envío
de instrucciones a la GPU. Usa tres cuadros de calentamiento y diez medidos por
vista. El encuadre gira 0.7 grados por cuadro.

Medición con Vader en Apple M1 (1200×644, calidad interactiva, Tatooine y todos los efectos activos):

| Vista | Tiempo medio, con lectura | Cuadros/s del benchmark |
|---|---:|---:|
| Principal | 15.2 ms | 65.8 |
| Motor | 24.2 ms | 41.4 |
| Cabina | 64.7 ms | 15.4 |
| Refracción | 44.8 ms | 22.3 |
| Superior | 14.3 ms | 69.7 |
| Droide | 23.6 ms | 42.4 |
| Vader | 32.6 ms | 30.7 |

Estas cifras son una medición, no un mínimo garantizado. Cambian con el encuadre
y la carga del equipo; la ventana limita la presentación a 60 FPS. En vidrio,
Nítido 1200 conserva resolución y puede quedar por debajo de 30 FPS.

### Motor CPU de respaldo

Comparación de CPU a 1200×644, todos los efectos activos y calidad interactiva:

| Vista | Versión inicial | BVH optimizada | Reducción de tiempo |
|---|---:|---:|---:|
| Principal | 115 ms | 70 ms | 39 % |
| Cabina | 750 ms | 360 ms | 52 % |

Medianas de cuatro ejecuciones por versión, alternadas en el mismo Apple M1 para
reducir el efecto de cambios de carga. La búsqueda optimizada utiliza una jerarquía
construida por superficie y cantidad de bloques (SAH), índices de hojas contiguos,
inversas de dirección precalculadas y distancias de cajas reutilizadas durante el
recorrido. No reduce resolución, muestras, materiales ni profundidad de los rayos.

Los tiempos corresponden al trazado en CPU, sin codificación PNG ni presentación.
Varían con el encuadre y la carga del sistema. Las vistas del vidrio aún son
costosas; Retina constante prioriza detalle y no garantiza movimiento fluido.

```sh
cargo run --release --offline -- --benchmark-sharp
cargo run --release --offline -- --benchmark
```

## Cumplimiento de la rúbrica

Ver [guion de presentación y evidencia por criterio](PRESENTACION.md).

| Apartado | Implementación y evidencia |
|---|---|
| Complejidad (30, subjetivo) | 5,902 bloques: casco circular escalonado, mandíbulas, cabina lateral, antena, torreta, ventiladores, rampa, puerto, droide, Vader y dos soldados rebeldes con disparos. |
| Atractivo visual (20, subjetivo) | Composición sobre base finita, materiales con textura, luz cálida y de relleno, sombras, oclusión local y suavizado de bordes. |
| Rotación y zoom (20) | Cámara orbital interactiva alrededor de un objetivo, control de azimut, elevación y distancia. Se recalculan los rayos al moverla. |
| Materiales (hasta 25) | Cinco materiales base más tres superficies para Vader; cada uno con textura y parámetros de albedo, especular, transparencia y reflectividad. Tabla siguiente. |
| Refracción (10) | Ley de Snell, índice 1.5 para vidrio, interfaces de entrada y salida, reflexión interna total. Vidrio en cabina y depósito; su interior permite observar el desplazamiento óptico. |
| Reflexión (5) | Rayos secundarios reflejados en metal pulido y vidrio, con mezcla de Fresnel para el material transparente. |
| Skybox (20) | Cubemap de seis caras, muestreo por dirección y filtrado bilineal. Entornos intercambiables: desierto con dos soles y espacio con estrellas y estación parcialmente destruida, visibles también en reflejos. |
| Entrega | Código fuente, capturas actuales y [video final con audio](renders/diorama-final.mp4) incluidos en el repositorio. |

La tabla documenta la implementación; las puntuaciones subjetivas corresponden
al evaluador. Los cinco materiales base son casco, metal oscuro, vidrio, arenisca
y motor. Cada uno tiene una textura procedural distinta y sus propios parámetros;
los tres materiales adicionales no aumentan el máximo de 25 puntos de ese apartado.

## Iluminación y presentación

Una luz principal cálida define el volumen y una luz de relleno azul conserva
los detalles de las caras en sombra. La exposición y la luz ambiente se ajustan
para distinguir paneles del casco, ventiladores y pasillo lateral. La plataforma
central tiene un tono más oscuro que la arena exterior y el motor mantiene una
banda azul emisiva. La escena contiene 5,902 bloques y tres luces: dos generales y una roja de alcance limitado junto al sable.

## Zona de mantenimiento

El astromecánico tiene cuerpo blanco con paneles azules, cúpula escalonada,
sensor frontal, patas laterales y tercer apoyo. Una consola con pantalla luminosa
y cable sobre la plataforma completa el área de servicio. Las piezas usan los
materiales existentes; no se añaden luces ni cálculos ópticos nuevos.

La tecla **6** abre una vista cercana. Desde ahí también se puede rotar y usar zoom.

![Droide y estación de mantenimiento](renders/droide.png)

## Vader, sable y entorno espacial

Vader ocupa un espacio propio junto al borde frontal de la base, separado de
las mandíbulas del Halcón. La vista **7** muestra el casco escalonado, respirador, panel del pecho, capa con
pliegues, botas y empuñadura estriada. La tela y la armadura tienen respuestas
ópticas distintas. El sable contiene bloques emisivos y un halo analítico
recortado por la distancia al primer impacto; la luz roja local utiliza sombras
y atenúa su intensidad hasta cero a tres unidades. El halo es un efecto artístico,
no una simulación de dispersión volumétrica ni iluminación global.

![Vader con sable rojo](renders/vader.png)

**C** cambia de entorno. El cielo espacial se genera por código en un cubemap de
768 píxeles por cara, con estrellas y una estación de combate parcialmente
destruida. No añade bloques a la BVH. Su iluminación de relleno es más fría.
El cielo de Tatooine se conserva y **B** permite desactivar cualquiera de ellos.

![Diorama en el espacio](renders/espacio.png)

## Soldados rebeldes y disparos

Dos soldados de bloques se ubican junto a la rampa lateral, separados del casco.
Tienen cascos blancos con franja oscura, camisa azul grisácea, chaleco negro,
cinturón, botas y bláster con mira. Reutilizan los materiales del diorama.

**8** abre su cámara y reproduce `BLASTER FX.wav` una vez, acompañado por cuatro
láseres: izquierda, derecha, izquierda, derecha según el encuadre inicial.
Los ataques visuales comienzan a los 0.045, 0.275, 0.505 y 0.735 s del efecto;
los proyectiles anteriores continúan su recorrido mientras sale el siguiente.
Volver a pulsar **8** o su botón reinicia cámara, ráfaga y sonido. Dentro de esta vista,
**.** (punto), decimal numérico o el botón **Disparar** alternan el soldado que
lanza un proyectil rojo con destello en la boca del arma y `DISPARO FX.wav`.
El disparo manual interrumpe la ráfaga y usa un único proyectil. Una nueva
pulsación de **.** reinicia el disparo manual y su sonido.
Al terminar el efecto vuelve cantina. Cambiar de cámara cancela la animación.

![Soldados junto a la rampa](renders/rebeldes.png)
![Disparo manual](renders/disparo.png)

La animación dura 0.68 s. CPU y GPU calculan el mismo segmento luminoso en el
espacio 3D, oculto por la geometría más cercana y visible en las reflexiones.
Es un resplandor artístico, sin simulación física de plasma ni daño a objetos.
Se actualizan unos pocos parámetros por cuadro; la geometría y la BVH permanecen
cargadas. En GPU conserva los modos de nitidez de Q; la fluidez del respaldo CPU
sigue dependiendo de su costo de renderizado.

Medición de la versión con rebeldes en Apple M1: cámara 8 a 1200×644, calidad interactiva,
todos los efectos y un disparo activo: **24.60 ms por cuadro (40.6 FPS)** de
media en diez cuadros tras calentamiento, incluyendo lectura de GPU. Es una
medición de renderizado; la ventana limita la presentación a 60 FPS y puede
variar según la carga del equipo.

El estado previo está guardado en la etiqueta `respaldo-antes-rebeldes-b5e459e`.
Los dos WAV nuevos, igual que los anteriores, son archivos locales excluidos de Git.

## Introducción

El prólogo comienza en silencio con «En una galaxia muy lejana…» en azul.
Cuatro segundos después aparece **MILLENIUM FALCON** y comienza `theme.wav`.
**EPISODIO II — UNIVERSO DE RAYOS** introduce una historia de androides,
rebeldes y un joven programador, con letras amarillas en perspectiva. El reloj
del audio gobierna el desplazamiento del texto; al terminar la pista entra el
diorama. Con la grabación local, la secuencia musical dura **90.112 segundos**
y la presentación completa **94.112 segundos** aproximadamente.

**Enter/Espacio** detiene el tema y salta al diorama con el efecto TIE completo;
**I** vuelve a iniciar la presentación y después regresa a la cámara principal.
La fuente se toma del sistema si está disponible, con la fuente integrada de
raylib como alternativa. Esta presentación pertenece a la ventana nativa.

![Texto en perspectiva](renders/intro.png)

### Música y efectos

Raylib reproduce un único stream a la vez, actualizado desde un hilo propio
para que un cuadro costoso no interrumpa la música. **M** silencia o reactiva
el audio y conserva su estado al cambiar de vista o repetir la intro.

| Situación | Secuencia de audio |
|---|---|
| Título inicial / `I` | `theme.wav` una vez, desde que aparece el título |
| Omitir intro | Detener tema → `TIE FX.wav` completo → cantina |
| Diorama | `cantina.wav` en bucle |
| `2` Motor | `MF FX.wav` una vez → cantina |
| `6` Droide | `R2D2 FX.wav` una vez → cantina |
| `7` Vader | `LS FX.wav` una vez → `dv theme.wav` en bucle |
| `8` Rebeldes | `BLASTER FX.wav` una vez → cantina |
| `.` / Disparar en cámara 8 | `DISPARO FX.wav` una vez → cantina |
| Repetir `2`, `6`, `7` u `8` | Reinicia el efecto correspondiente |
| Salir de Vader con otra cámara, `R` o `I` | Detiene su música y aplica la nueva secuencia |

Los botones de las cámaras hacen lo mismo que sus teclas. Los archivos locales
se colocan en `assets/audio/` con los nombres exactos de la tabla y se excluyen
de Git. Consultar [preparación del audio](assets/audio/README.md) al copiar el
proyecto a otra computadora. Sin los archivos o un dispositivo de audio, la
presentación conserva la secuencia en silencio.

```sh
cargo run --release --offline -- --audio-check
```

Esta comprobación silenciada carga los catorce WAV, adelanta los streams cerca
de su final y verifica transiciones, bucles y mezcla de efectos de combate. La reproducción normal
respeta la duración completa de cada grabación.

Los tres materiales de Vader enriquecen el aspecto visual; la rúbrica limita la
puntuación por materiales a cinco, aunque la escena incluya ocho.

## Materiales

Los colores se calculan en espacio lineal. Los valores RGB de albedo están entre
0 y 1. Un cero en transparencia o reflectividad es intencional para materiales
opacos o mates. Las variantes de color del casco no se cuentan como materiales extra.

| Material | Textura propia | Albedo RGB | Especular / exponente | Transparencia | Reflectividad | IOR |
|---|---|---|---|---|---|---|
| Aleación del casco | Paneles, uniones, pernos y desgaste | 0.58, 0.61, 0.64 | 0.38 / 55 | 0 | 0.10 | 1.0 |
| Metal oscuro pulido | Estrías y grano de metal | 0.105, 0.14, 0.17 | 0.80 / 120 | 0 | 0.42 | 1.0 |
| Vidrio de cabina | Variación superficial fina con tinte azulado | 0.63, 0.83, 0.90 | 0.95 / 180 | 0.86 | 0.08 | 1.5 |
| Arenisca | Grano y variación por bloque | 0.61, 0.39, 0.20 | 0.06 / 12 | 0 | 0 | 1.0 |
| Paneles del motor | Franjas y celdas luminosas | 0.12, 0.66, 0.95 | 0.65 / 85 | 0 | 0.16 | 1.0 |
| Tejido de la capa | Trama de fibras | 0.022, 0.027, 0.038 | 0.09 / 8 | 0 | 0.01 | 1.0 |
| Armadura negra | Acabado fino con grano | 0.045, 0.052, 0.068 | 0.65 / 180 | 0 | 0.09 | 1.0 |
| Plasma del sable | Bandas de energía | 1.0, 0.035, 0.018 | 0.20 / 60 | 0 | 0.02 | 1.0 |

Las ocho texturas son procedurales y deterministas: se generan por código, con
muestreo de coordenadas de cada cara. El motor además tiene emisión para dar
apariencia luminosa; no se simula iluminación global por emisión.

![Motor posterior](renders/motor.png)

![Cabina con vidrio](renders/cabina.png)

### Comparación de refracción

| Activada | Desactivada |
|---|---|
| ![Vidrio con refracción](renders/refraccion.png) | ![Vidrio sin refracción](renders/refraccion-sin.png) |

## Dependencias

La única dependencia directa es **raylib 6**, utilizada para ventana, entrada, reproducción de audio,
texto, ejecución del shader propio y presentación de la imagen. Sus dependencias transitivas quedan registradas
en `Cargo.lock`.

El uso de raylib en esta implementación fue revisado y autorizado por el docente.

**Rust estándar** construye geometría, materiales, cámara, cubemap y BVH; también
implementa el raytracer CPU paralelo y el codificador PNG. **GLSL** implementa el
raytracer GPU con los mismos parámetros. Reflexión y refracción usan una pila
iterativa acotada en el shader, equivalente a la recursión del motor CPU.
No se utilizan motores 3D ni librerías matemáticas o de trazado de rayos.

Se conserva un visor web **opcional**, útil para grabar el video con Canvas 2D y
MediaRecorder nativos. No es necesario para presentar en la ventana:

```sh
cargo run --release --offline -- --web
```

Abrir `http://127.0.0.1:7878`. El servidor solo escucha localmente; `Ctrl+C` lo
cierra. `--port 7879` permite elegir otro puerto. El navegador muestra los píxeles
calculados por Rust y graba solo su canvas, sin cámara ni micrófono.

## Renderizar sin navegador

```sh
cargo run --release --offline -- --render
```

Este comando usa CPU y produce `renders/falcon.png`. Para exportar con GPU:

```sh
cargo run --release --offline -- --gpu-check --width 1600 --height 1050 --quality 2 --output renders/gpu-check.png
```

La exportación GPU requiere una sesión gráfica aunque la ventana de prueba esté
oculta. Ambos motores aceptan los mismos parámetros de cámara y efectos.
 Se pueden ajustar resolución, calidad y cámara:

```sh
cargo run --release --offline -- --render --width 1600 --height 1050 --quality 2 --yaw 38 --pitch 22 --distance 25 --output renders/falcon-hd.png
```

`--quality 0`: rápida; `1`: equilibrada; `2`: cuatro muestras por píxel.
`--tx`, `--ty` y `--tz` cambian el punto que mira la cámara.
`--no-reflections`, `--no-refractions` y `--no-skybox` permiten comparar efectos.

Para exportar las seis texturas del skybox:

```sh
cargo run --release --offline -- --render --skybox
```

Las caras `skybox-0.png` a `skybox-5.png` corresponden a +X, −X, +Y, −Y, +Z y −Z.
Para exportar un recorrido como secuencia PNG:

```sh
cargo run --release --offline -- --tour 120 --width 720 --height 470 --quality 1
```

En el visor web opcional, el botón **Grabar video del diorama** crea directamente `renders/diorama.webm`.
Si un navegador no admite WebM/MediaRecorder, el resto del visor sigue funcionando;
la grabación requiere uno que sí los admita.

## Cómo funciona

1. Se construyen cubos y prismas para la nave y el puerto. Una jerarquía de cajas
   envolventes (BVH) construida con SAH reduce las intersecciones necesarias por
   rayo y recorre primero las cajas más cercanas.
2. La cámara orbital genera rayos en perspectiva. Se toma el impacto positivo
   más cercano mediante el método de intervalos por eje.
3. Se muestrea la textura del material en la cara alcanzada y se calcula luz
   ambiente, difusa y especular. Rayos hacia las luces producen sombras, con
   transmisión aproximada a través del vidrio.
4. Se generan rayos secundarios para reflexión y refracción. Fresnel distribuye
   la parte transparente entre ambos efectos. La profundidad y la
   contribución se limitan para controlar el tiempo de render. En GPU se usa una
   pila explícita, con un máximo de 127 tareas para seis niveles secundarios.
5. Los rayos que no tocan objetos muestrean el cubemap. La imagen final recibe
   mapeo de tonos y corrección gamma, y se presenta directamente en raylib. La exportación PNG utiliza código propio.

`src/scene.rs` construye el diorama; `material.rs` define superficies;
`geometry.rs` contiene cajas y BVH; `render.rs` traza los rayos;
`camera.rs` maneja la cámara; `skybox.rs` genera y muestrea el cubemap;
`png.rs` codifica la imagen; `viewer.rs` contiene la ventana, el renderizado
asíncrono y la resolución adaptativa; `gpu.rs` carga los datos y el shader;
`shaders/raytrace.fs` calcula la imagen GPU; `intro.rs` y `shaders/crawl.fs` muestran el prólogo; `audio.rs` coordina música y efectos; `server.rs` conserva el visor web opcional.

## Verificar

```sh
cargo test --offline
cargo clippy --offline --all-targets -- -D warnings
cargo tree --offline
cargo run --release --offline -- --validate-gpu
```

Verificación de entrega del 1 de octubre de 2026: **29 pruebas aprobadas**,
formato y Clippy sin advertencias, **64 comparaciones CPU/GPU del diorama** y
**34 de vuelo y combate** aprobadas. Los **14 audios**, sus transiciones, bucles
y efectos simultáneos están verificados.

Las pruebas cubren Snell, reflexión interna total, entrada/salida del vidrio,
rayos dentro y fuera de cajas, equivalencia BVH/búsqueda completa, cámara orbital,
continuidad del cubemap, límites físicos de los materiales y cancelación/reanudación
del renderizado. También comparan la jerarquía con una búsqueda exhaustiva en
la escena completa: rayos paralelos, impactos rasantes, orígenes dentro de bloques,
límites de distancia y geometría coincidente. La prueba de empaquetado verifica
que la BVH exportada para GPU conserva todos los bloques y los impactos.

`--validate-gpu` necesita un contexto gráfico y compara 64 imágenes: ocho vistas,
calidad interactiva/final, los tres efectos desactivados por separado, el cielo espacial
y dos instantes del disparo. Informa
error medio por canal y proporción de píxeles con diferencia mayor que 32/255.
La tolerancia es error medio ≤1/255 y como máximo 0.5 % de píxeles por encima de
ese umbral, para admitir diferencias de redondeo en aristas compartidas.

## Referencia visual

[Millennium Falcon — Star Wars Databank](https://www.starwars.com/databank/Millennium-Falcon).
Interpretación académica en bloques. Geometría, texturas y entorno generados en
este proyecto; no se incorporaron modelos ni texturas descargadas.
