# Prueba de imagen y fluidez

Rama `mejora-imagen-fluida`, basada en la entrega `0c6c051`. Añade un filtro
de bordes en pantalla y una opción de 1600 px en el modo nave. Inicia a 1200 px.
No añade dependencias. `J` permite comparar el filtro y `Q` alterna
1200/1600/800 en vuelo. Si el shader no está disponible, se presenta la imagen
sin filtrar. El respaldo CPU conserva su comportamiento anterior.

El filtro reduce escalones en siluetas y diagonales, pero puede suavizar detalles
finos; no reemplaza el muestreo de varios rayos por píxel. El refinamiento en
reposo del diorama sigue disponible. No cambia las capturas ni los videos
exportados, que se obtienen directamente del raytracer.

## Medición reproducible

```sh
cargo run --release --offline --locked --example benchmark_display
```

Apple M1, salida fija de 1600×1000, una muestra por píxel y cámara en movimiento.
Cada configuración procesa 44 cuadros; se descartan los primeros 12 y se miden
32. El tiempo incluye raytracing, presentación y lectura sincronizada de la GPU.
Por ello no equivale al contador FPS de la ventana. Las imágenes comparativas
se guardan en `/tmp/calidad-*.png`.

| Escena | Ancho | Suavizado | Mediana | Peor cuadro |
|---|---:|---|---:|---:|
| Diorama | 1200 | No | 19.11 ms | 21.64 ms |
| Diorama | 1200 | Sí | 23.38 ms | 31.55 ms |
| Diorama | 1600 | Sí | 28.19 ms | 33.17 ms |
| Espacio con 3 TIE | 1200 | No | 8.73 ms | 10.83 ms |
| Espacio con 3 TIE | 1200 | Sí | 8.70 ms | 11.14 ms |
| Espacio con 3 TIE | 1600 | Sí | 14.92 ms | 22.50 ms |
| Espacio con 3 TIE e impulso | 1200 | No | 7.83 ms | 9.64 ms |
| Espacio con 3 TIE e impulso | 1200 | Sí | 9.24 ms | 11.13 ms |
| Espacio con 3 TIE e impulso | 1600 | Sí | 13.30 ms | 16.18 ms |

Una repetición de la referencia de 1200 sin filtro dio medianas de 20.04, 9.52
y 9.03 ms respectivamente. Hay variación por carga y frecuencia de la GPU;
estos resultados cortos no garantizan 60 FPS constantes. En la ventana real (2400×1664 de captura, viewport de vuelo 1600×834),
1600 con filtro marcó 34 FPS con TIE e impulso; 1200 con filtro marcó 61 FPS.
Son lecturas puntuales del contador, no promedios sostenidos. Se verificaron
los tres niveles de Q, el cambio de J y un impacto con F. La prueba de ventana
justifica conservar 1200 como ancho inicial del vuelo; 1600 queda opcional.
El diorama también conserva 1200 durante el movimiento en su modo inicial. Para priorizar rendimiento se puede desactivar
el filtro con `J` y bajar resolución con `Q`.

Validación: compilación release, 29 pruebas y Clippy con todos los targets y
advertencias tratadas como errores. El shader se compiló y se compararon sus
imágenes durante el benchmark.
