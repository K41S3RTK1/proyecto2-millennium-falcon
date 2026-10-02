# Guion de presentación

## Calidad recomendada

En las vistas 1–8, `Q` recorre Nítido 1200 → Detalle 1600 → Fluido adaptativo
→ Retina constante. `J` activa el suavizado en GPU. La calidad seleccionada se
conserva al cambiar de vista.

- **Máxima nitidez en movimiento:** Retina, con mayor costo. No implica mayor FPS.
- **Exposición de detalles:** Detalle 1600 con J activado es un punto de partida;
  detener la cámara permite que el refinamiento automático alcance resolución
  Retina y cuatro muestras por píxel, incluso si se estaba usando Nítido 1200.
- **Rotación, zoom y efectos:** Nítido 1200 para priorizar fluidez. Si una vista
  de vidrio resulta lenta, usar Fluido adaptativo. El costo varía con el encuadre.
- **Modo 9:** Q recorre 1200 → 1600 → 800. Usar 1200 para combate; 1600 ofrece
  más detalle pero en la prueba de TIE con impulso bajó a unos 34 FPS.

El contador UI FPS no equivale a imágenes nuevas del raytracer. En reposo es
normal ver cero imágenes/s después del refinamiento. Para comparar texturas
finas se puede alternar J: el suavizado de bordes puede suavizar algo su detalle.

## Orden para demostrar la rúbrica

Empezar por los requisitos y reservar los extras para el cierre. Si el tiempo
es corto, saltar la intro con Enter/Espacio; el video conserva la versión completa.

| Criterio | Demostración | Frase sugerida |
|---|---|---|
| Complejidad, 30 puntos subjetivos | 1, 5, 2, 6, 7 y 8: nave, puerto, motor, droide y personajes | «El diorama contiene 5902 bloques, con detalles de casco, cabina, rampa, motor y personajes. Una BVH acelera la búsqueda de intersecciones.» |
| Atractivo visual, 20 puntos subjetivos | Vista 1 quieta y motor 2; enseñar composición, iluminación y materiales | «Busqué una escena reconocible de Star Wars, con iluminación cálida del puerto y acentos de luz azul y roja.» |
| Rotación y acercamiento, 20 puntos | Girar con arrastre/flechas y acercar/alejar con rueda o +/− | «Puedo inspeccionar la escena desde diferentes ángulos y distancias en tiempo real.» |
| Cinco materiales, hasta 25 puntos | Mostrar casco, metal oscuro, vidrio, arenisca y motor; abrir la tabla del README si hace falta | «Cada material tiene su textura procedural y parámetros propios de albedo, especular, transparencia y reflectividad; no son únicamente cambios de color.» |
| Refracción, 10 puntos | 3 y especialmente 4; comparar G apagado/encendido sin cambiar el encuadre | «Los rayos cambian de dirección al atravesar el vidrio, con índice de refracción 1.5. El vidrio de cabina tiene sentido dentro de la escena.» |
| Reflexión, 5 puntos | Comparar F en metal pulido o vidrio y mover ligeramente la cámara | «Los materiales reflectantes lanzan rayos secundarios hacia el entorno. Esto es distinto del brillo especular de una luz.» |
| Skybox, 20 puntos | C para alternar Tatooine/espacio; B para apagar/encender el cielo; girar | «El entorno es un cubemap de seis caras, consultado según la dirección del rayo, y participa en los reflejos.» |
| Entrega | Abrir repositorio, README y enlace al MP4 | «Aquí están el código, instrucciones y video del diorama.» |

Dejar F, G y B activados después de las comparaciones. En el modo 9, F cambia
de función y dispara a los TIE; las comparaciones de reflexión se hacen en 1–8.
La luz roja cercana a Vader incluye iluminación local: por sí sola no demuestra
reflexión. El resplandor del sable tampoco demuestra refracción.

Para los materiales, destacar cinco ejemplos verificables en `src/material.rs`:

| Material | Especular | Transparencia | Reflectividad | Rasgo visible |
|---|---:|---:|---:|---|
| Aleación del casco | 0.38 | 0 | 0.10 | Paneles y uniones |
| Metal oscuro pulido | 0.80 | 0 | 0.42 | Superficie más reflectante |
| Vidrio de la cabina | 0.95 | 0.86 | 0.08 | Transmisión y refracción, IOR 1.5 |
| Arenisca del puerto | 0.06 | 0 | 0 | Textura de bloques y grano |
| Paneles del motor | 0.65 | 0 | 0.16 | Textura y emisión celeste |

Los albedos RGB y las ocho texturas están documentados en el código y README.
Que varios materiales opacos tengan transparencia cero no elimina su parámetro
propio. Hay ocho materiales, pero el apartado puntúa un máximo de cinco.

## Cierre y extras

Mostrar 8: cuatro disparos izquierda/derecha/izquierda/derecha con el sonido
de entrada. Punto permite un disparo manual alternado. Si queda tiempo, 9 muestra
el ascenso de unos 50 segundos; después E activa el impulso y F combate contra
tres TIE, cada uno con tres impactos. Presentarlo como interacción adicional.

Frase de cierre: «Además de las propiedades ópticas de los materiales, integré
dos entornos, personajes y una escena de vuelo con combate, manteniendo el
raytracing propio para calcular la imagen.»

El docente ya autorizó raylib en esta implementación. Explicar su función con
precisión: ventana, entrada, audio y ejecución del shader; las intersecciones,
BVH, materiales, sombras, reflexión y refracción son código del proyecto.

Los criterios técnicos tienen evidencia en el proyecto; complejidad y atractivo
siguen sujetos a evaluación. Los puntos escritos en el enunciado suman **130**,
aunque se anuncia una nota máxima de 100: confirmar con el docente cómo se
convierten a porcentaje, sin asumir una normalización. El techo por boleto es
80 sin boleto, 90 con plata y 100 con oro; el boleto no sustituye el cumplimiento.

El video de 4:49 ya incluye el vuelo y combate; conserva la versión previa al
suavizado y a la ráfaga automática. Demostrar estas mejoras en vivo.

## Sable interactivo de Vader

Desarrollado en `extra-sable-interactivo` e integrado en `main`. En la vista
7, `L` o el botón Sable enciende/apaga la hoja sin añadir ni reiniciar audio.
La empuñadura permanece visible; se controlan conjuntamente la hoja de 22 bloques,
su emisión y halo, y la luz roja local. El estado se conserva al cambiar de cámara.
El cálculo CPU/GPU excluye la hoja apagada de los rayos primarios, las sombras,
la oclusión, los reflejos y los rayos que atraviesan el vidrio. No reconstruye la
geometría ni la BVH por cuadro. Al iniciar la aplicación el sable está encendido.

El plasma actual es opaco y emisivo; no refracta. Su imagen puede aparecer en
materiales reflectantes y puede verse a través de vidrio refractivo cuando el
encuadre lo permite. La refracción de la rúbrica se demuestra claramente con
el vidrio de las vistas 3–4; no exige que el sable reúna todos los efectos.

## Cómo explicar los tres efectos

**Refracción:** la luz cambia de dirección al cruzar entre medios con distintos
índices de refracción. En esta escena los rayos entran y salen del vidrio de la
cabina y del depósito, cuyo índice es 1.5. El vidrio permite mirar al interior,
pero desplaza la imagen según el ángulo; no es solamente bajar la opacidad.
Tiene sentido contextual porque son superficies de vidrio de una nave y de
un recipiente transparente. Mostrar las vistas 3–4 y comparar con G.

En la vista 4 hay **tres barras celestes reales dentro del depósito**, construidas
con bloques de material emisivo. No son imágenes de los postes exteriores ni
aparecen porque la refracción multiplique una luz. Lo que cambia con la
refracción es la trayectoria del rayo y la posición aparente de los objetos
vistos a través del vidrio. Girar lentamente permite observar ese desplazamiento
al mirar por caras y ángulos distintos.

En esta implementación G desactiva la transmisión refractiva completa: el vidrio
se vuelve opaco y oculta las barras. Es una comparación del efecto activado y
desactivado, no una comparación entre vidrio que desvía rayos y vidrio igualmente
transparente que los deja pasar rectos. Las barras siguen dentro en ambos casos.

Frase para la vista 4: «Este depósito contiene tres barras luminosas. Las veo a
través de un material de vidrio con índice 1.5; el raytracer desvía los rayos
al atravesar su superficie y cambia su posición aparente según el ángulo.
Al desactivar G, desactivo esa transmisión y el vidrio oculta el interior».

Frase: «El vidrio tiene un índice óptico diferente del aire: calculo cómo se
curvan los rayos al atravesarlo, por eso cambia lo que vemos detrás».

**Reflexión:** la luz rebota en una superficie. El raytracer lanza un rayo
secundario en la dirección reflejada y combina su color según el material.
El metal oscuro pulido tiene reflectividad 0.42; el casco y el vidrio también
reflejan en menor proporción, con Fresnel en el vidrio. Tiene sentido contextual
porque una nave tiene superficies metálicas y ventanas que reflejan su entorno.
Comparar F en una vista cercana de metal/vidrio y variar un poco el ángulo.
El brillo especular o la luz roja cercana al sable, por sí solos, no prueban
que se esté reflejando otra parte de la escena.

Frase: «El material devuelve parte de la luz del entorno; el reflejo cambia
cuando muevo la cámara porque recalculo la dirección del rayo reflejado».

**Skybox:** un entorno que rodea la escena, representado aquí con seis caras de
un cubemap. Cuando un rayo no golpea geometría, su dirección determina el color
que toma del cielo; esto también se aplica a rayos de reflexión que llegan al
entorno. No es una propiedad óptica equivalente a transparencia o reflectividad,
sino el entorno que pueden ver esos materiales. Los dos cielos tienen sentido:
Tatooine sitúa el puerto y el espacial sitúa el vuelo del Halcón. Mostrar C para
cambiar el entorno, B para desactivarlo/activarlo y girar para ver su continuidad.

Frase: «La escena está rodeada por un cubemap de seis caras. El cielo cambia
según la dirección de observación y también puede verse en los reflejos».
