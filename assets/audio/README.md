# Audio local

Copiar los nueve WAV en esta carpeta, conservando exactamente estos nombres:

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

Las grabaciones locales son WAV PCM, 48 kHz, estéreo, 24 bits. Raylib lee su
duración real; no se recortan ni convierten. Repetir una tecla de efecto reinicia
su secuencia. Nunca se reproducen dos pistas simultáneamente. M silencia todo
sin alterar el avance; I detiene la pista actual y repite la intro desde su fase
azul silenciosa. Al cerrar se detiene y descarga el audio.

Los WAV locales están excluidos de Git. Al descargar el repositorio en otro
equipo es necesario copiarlos por separado. Sin archivos, la aplicación sigue
funcionando en silencio. `cargo run --release --offline -- --audio-check` permite
verificar carga, transiciones y bucles sin reproducir sonido audible.
