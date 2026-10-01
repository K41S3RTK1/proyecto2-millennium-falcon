# Audio local

Copiar los catorce WAV en esta carpeta, conservando exactamente estos nombres:

| Archivo | Duración de la grabación local | Uso |
|---|---:|---|
| `theme.wav` | 90.112 s | Intro, una vez desde el título |
| `cantina.wav` | 90.112 s | Ambiente en bucle |
| `dv theme.wav` | 54.613 s | Música de Vader en bucle |
| `TIE FX.wav` | 4.096 s | Omitir intro, una vez antes de cantina |
| `LS FX.wav` | 13.653 s | Vista 7, una vez antes del tema de Vader |
| `R2D2 FX.wav` | 3.413 s | Vista 6, una vez antes de cantina |
| `MF FX.wav` | 8.192 s | Vista 2, una vez antes de cantina |
| `BLASTER FX.wav` | 2.048 s | Vista 8, una vez antes de cantina |
| `DISPARO FX.wav` | 0.683 s | Cada disparo manual en vista 8 |
| `Despegue.wav` | 49.835 s | Ascenso del modo 9, una vez y completo |
| `ambientefalcon.wav` | 97.621 s | Ambiente espacial del modo 9, en bucle |
| `Impulso.wav` | 19.797 s | E en el espacio, una vez; después regresa el ambiente |
| `disparofalcon.wav` | 2.048 s | Cada disparo aceptado con F en el espacio |
| `explotion.wav` | 3.413 s | Cada TIE destruido al tercer impacto |

Las grabaciones originales son WAV PCM, 48 kHz, estéreo, 24 bits.
Los tres archivos del modo nave también son WAV a 48 kHz; despegue e impulso
son mono y el ambiente espacial es estéreo. Raylib lee su
duración real; no se recortan ni convierten. Repetir una tecla de efecto reinicia
su secuencia. Hay una sola pista principal. En combate, disparos y explosiones
suenan encima del ambiente o impulso, sin cortarlos. Se usan cuatro voces de
disparo y tres de explosión para conservar las colas de efectos consecutivos.
Los dos efectos nuevos son PCM estéreo de 48 kHz y 24 bits. M silencia todo
sin alterar el avance; I detiene la pista actual y repite la intro desde su fase
azul silenciosa. Al cerrar se detiene y descarga el audio.

Los WAV locales están excluidos de Git. Al descargar el repositorio en otro
equipo es necesario copiarlos por separado. Sin archivos, la aplicación sigue
funcionando en silencio. `cargo run --release --offline -- --audio-check` permite
verificar carga, transiciones y bucles sin reproducir sonido audible.

El reloj del despegue gobierna el ascenso. E se habilita al terminarlo; cada
pulsación interrumpe el ambiente y reinicia el impulso. Salir con 1–8 cancela
la pista de vuelo y activa el audio de la vista elegida. 9 reinicia todo el
ascenso. Los audios se conservan completos y sin conversiones.

N detiene las colas de combate al reponer la oleada. Salir con 1–8, repetir
el ascenso con 9 o cerrar la ventana también detiene los efectos.
