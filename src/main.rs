use falcon_diorama::{
    camera::Camera,
    png,
    render::{self, Settings},
    scene::Scene,
    server, viewer,
};
use std::{error::Error, path::PathBuf};
fn value(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|s| s == name)
        .and_then(|i| args.get(i + 1))
        .cloned()
}
fn main() -> Result<(), Box<dyn Error>> {
    // Finder inicia la app sin el directorio del proyecto; el bundle vive en target/.
    if let Ok(exe) = std::env::current_exe()
        && exe.parent().is_some_and(|p| p.ends_with("Contents/MacOS"))
        && let Some(project) = exe.ancestors().nth(5)
    {
        std::env::set_current_dir(project)?;
    }
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|s| s == "--help") {
        println!(
            "Millennium Falcon · Raytracing Rust + ventana raylib\n\ncargo run --release                 Ventana nativa (recomendado para Mac M1)\ncargo run --release -- --render     Guarda renders/falcon.png\nOpciones: --width 1100 --height 720 --quality 2 --yaw 38 --pitch 29 --distance 23 --output ruta.png\n--flight-preview --flight-progress 1 --boost-age 1 Exporta el modo nave\n--combat-preview Exporta tres TIE (--combat-shot-age .3 / --explosion-age .4)\n--validate-flight Compara el modo nave CPU/GPU\n--space     Selecciona el cielo espacial\n--audio-check Verifica los catorce audios y sus transiciones\n--intro-frame 45 Exporta un cuadro de la intro (--output ruta.png)\n--cpu       Inicia con raytracing CPU (T alterna CPU/GPU)\n--gpu-check Exporta un cuadro GPU; necesita contexto gráfico\n--validate-gpu Compara 64 imágenes CPU/GPU\n--benchmark-gpu Mide ocho vistas GPU, con lectura sincronizada\n--adaptive  Inicia con resolucion adaptativa (Q cambia el modo)\n--benchmark-sharp Compara nitidez fija a 1200 y 2400 px\n--demo      Abre la ventana con recorrido automatico\n--web       Visor web opcional y grabacion de video\n--benchmark Mide renderizado interactivo sin abrir ventana\n--port 7878  Puerto del visor web\n--tour 120   Exporta 120 imágenes para video\n--skybox     Exporta las seis caras del cubemap\n--no-reflections --no-refractions --no-skybox  Comparaciones"
        );
        return Ok(());
    }
    if let Some(v) = value(&args, "--intro-frame") {
        let seconds: f32 = v.parse()?;
        if !seconds.is_finite()
            || !(0.0..=falcon_diorama::intro::DEFAULT_DURATION).contains(&seconds)
        {
            return Err("Tiempo de intro fuera de rango".into());
        }
        return falcon_diorama::intro::snapshot(
            seconds,
            &value(&args, "--output").unwrap_or_else(|| "/tmp/falcon-intro.png".into()),
        );
    }
    if args.iter().any(|v| v == "--audio-check") {
        return falcon_diorama::audio::check();
    }
    let combat_preview = args.iter().any(|s| s == "--combat-preview");
    let flight_preview = combat_preview || args.iter().any(|s| s == "--flight-preview");
    let mut camera = Camera::default();
    let mut settings = Settings::default();
    if flight_preview {
        settings.flight.active = true;
        settings.flight.progress = value(&args, "--flight-progress")
            .unwrap_or_else(|| "1".into())
            .parse()?;
        settings.flight.boost_age = value(&args, "--boost-age")
            .unwrap_or_else(|| "-1".into())
            .parse()?;
        if !settings.flight.progress.is_finite()
            || !(0.0..=1.0).contains(&settings.flight.progress)
            || !settings.flight.boost_age.is_finite()
        {
            return Err("Estado de vuelo inválido".into());
        }
        settings.combat.enabled = combat_preview && settings.flight.progress >= 1.;
        if let Some(age) = value(&args, "--combat-shot-age") {
            settings.combat.shot_age = age.parse()?;
            if !settings.combat.shot_age.is_finite() {
                return Err("Tiempo inválido".into());
            }
        }
        if let Some(age) = value(&args, "--explosion-age") {
            let age: f32 = age.parse()?;
            if !age.is_finite() || age < 0. {
                return Err("Tiempo inválido".into());
            }
            settings.combat.hp[0] = 0;
            settings.combat.impacts[0] = age;
        }
        camera = settings.combat.camera(settings.flight);
    }
    if let Some(v) = value(&args, "--shot-age") {
        settings.shot.age = v.parse()?;
        if !settings.shot.age.is_finite() {
            return Err("Tiempo de disparo inválido".into());
        }
    }
    if let Some(v) = value(&args, "--width") {
        settings.width = v.parse()?;
    }
    if let Some(v) = value(&args, "--height") {
        settings.height = v.parse()?;
    }
    if let Some(v) = value(&args, "--quality") {
        settings.quality = v.parse()?;
    }
    if let Some(v) = value(&args, "--yaw") {
        camera.yaw = v.parse()?;
    }
    if let Some(v) = value(&args, "--pitch") {
        camera.pitch = v.parse()?;
    }
    if let Some(v) = value(&args, "--distance") {
        camera.distance = v.parse()?;
    }
    if let Some(v) = value(&args, "--tx") {
        camera.target.x = v.parse()?;
    }
    if let Some(v) = value(&args, "--ty") {
        camera.target.y = v.parse()?;
    }
    if let Some(v) = value(&args, "--tz") {
        camera.target.z = v.parse()?;
    }
    if settings.width == 0
        || settings.height == 0
        || settings.width > 4096
        || settings.height > 4096
        || settings.quality > 2
        || !camera.target.x.is_finite()
        || !camera.target.y.is_finite()
        || !camera.target.z.is_finite()
        || !camera.yaw.is_finite()
        || !camera.pitch.is_finite()
        || !camera.distance.is_finite()
        || camera.pitch.abs() > 85.
        || camera.distance < 3.
    {
        return Err("Dimensiones o cámara fuera de rango".into());
    }
    settings.space = flight_preview || args.iter().any(|v| v == "--space");
    settings.reflections = !args.iter().any(|v| v == "--no-reflections");
    settings.refractions = !args.iter().any(|v| v == "--no-refractions");
    settings.skybox = !args.iter().any(|v| v == "--no-skybox");
    let scene = if flight_preview {
        Scene::flight_combat(settings.combat.mask())
    } else {
        Scene::new()
    };
    println!(
        "{} bloques · {} materiales · BVH · Rust estándar",
        scene.blocks.len(),
        scene.materials.len()
    );
    std::fs::create_dir_all("renders")?;
    if args.iter().any(|v| v == "--skybox") {
        let (faces, size) = scene.sky.faces();
        for (i, face) in faces.iter().enumerate() {
            let mut pixels = Vec::new();
            for &v in face {
                let c = render::display(v);
                pixels.extend_from_slice(&[c[0], c[1], c[2], 255]);
            }
            png::save(format!("renders/skybox-{i}.png"), size, size, &pixels)?;
        }
    }
    if args.iter().any(|v| v == "--validate-flight") {
        falcon_diorama::gpu::validate_flight()?;
    } else if args.iter().any(|v| v == "--validate-gpu") {
        falcon_diorama::gpu::validate(&scene)?;
    } else if args.iter().any(|v| {
        v == "--gpu-check"
            || v == "--benchmark-gpu"
            || (flight_preview && !args.iter().any(|s| s == "--render"))
    }) {
        falcon_diorama::gpu::check(
            &scene,
            camera,
            settings,
            &value(&args, "--output").unwrap_or_else(|| "renders/gpu-check.png".into()),
            args.iter().any(|v| v == "--benchmark-gpu"),
        )?;
    } else if args.iter().any(|v| v == "--benchmark-sharp") {
        viewer::benchmark_sharp(&scene);
    } else if args.iter().any(|v| v == "--benchmark") {
        viewer::benchmark(&scene);
    } else if let Some(v) = value(&args, "--tour") {
        let count: usize = v.parse()?;
        if count == 0 || count > 3600 {
            return Err("El tour admite de 1 a 3600 cuadros".into());
        }
        std::fs::create_dir_all("renders/tour")?;
        for i in 0..count {
            camera.yaw = 38. + i as f32 * 360. / count as f32;
            camera.distance = 23. - 2. * (i as f32 / count as f32 * std::f32::consts::TAU).sin();
            let frame = render::render(&scene, camera, settings);
            png::save(
                format!("renders/tour/{i:04}.png"),
                frame.width,
                frame.height,
                &frame.pixels,
            )?;
            println!(
                "Cuadro {}/{count}: {:.2}s",
                i + 1,
                frame.elapsed.as_secs_f32()
            );
        }
    } else if args.iter().any(|v| v == "--render") {
        let frame = render::render(&scene, camera, settings);
        let path =
            PathBuf::from(value(&args, "--output").unwrap_or_else(|| "renders/falcon.png".into()));
        png::save(&path, frame.width, frame.height, &frame.pixels)?;
        println!(
            "{} · {}×{} · {:.2}s",
            path.display(),
            frame.width,
            frame.height,
            frame.elapsed.as_secs_f32()
        );
    } else if args.iter().any(|v| v == "--web") {
        let port = value(&args, "--port")
            .map(|v| v.parse::<u16>())
            .transpose()?
            .unwrap_or(7878);
        server::serve(scene, port)?;
    } else {
        viewer::run(
            scene,
            camera,
            settings,
            args.iter().any(|v| v == "--demo"),
            args.iter().any(|v| v == "--adaptive"),
            !args.iter().any(|v| v == "--cpu"),
        )?;
    }
    Ok(())
}
