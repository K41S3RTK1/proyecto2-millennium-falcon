//! Un único flujo audible. La decodificación se actualiza en un hilo propio para
//! que los renders de calidad no interrumpan el audio.
use raylib::prelude::*;
use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
    time::{Duration, Instant},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Track {
    Theme,
    Cantina,
    Vader,
    Tie,
    Saber,
    Droid,
    Falcon,
    Blaster,
    Shot,
    Takeoff,
    FlightAmbient,
    Boost,
}
impl Track {
    pub const ALL: [Self; 12] = [
        Self::Theme,
        Self::Cantina,
        Self::Vader,
        Self::Tie,
        Self::Saber,
        Self::Droid,
        Self::Falcon,
        Self::Blaster,
        Self::Shot,
        Self::Takeoff,
        Self::FlightAmbient,
        Self::Boost,
    ];
    pub fn filename(self) -> &'static str {
        match self {
            Self::Theme => "theme.wav",
            Self::Cantina => "cantina.wav",
            Self::Vader => "dv theme.wav",
            Self::Tie => "TIE FX.wav",
            Self::Saber => "LS FX.wav",
            Self::Droid => "R2D2 FX.wav",
            Self::Falcon => "MF FX.wav",
            Self::Blaster => "BLASTER FX.wav",
            Self::Shot => "DISPARO FX.wav",
            Self::Takeoff => "Despegue.wav",
            Self::FlightAmbient => "ambientefalcon.wav",
            Self::Boost => "Impulso.wav",
        }
    }
    pub fn looping(self) -> bool {
        matches!(self, Self::Cantina | Self::Vader | Self::FlightAmbient)
    }
    fn volume(self) -> f32 {
        if matches!(
            self,
            Self::Theme | Self::Cantina | Self::Vader | Self::FlightAmbient
        ) {
            0.45
        } else {
            0.65
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Event {
    BeginIntro,
    StartTheme,
    SkipIntro,
    FinishIntro,
    View(usize),
    Fire,
    Boost,
    Finished(Track),
    ToggleMute,
    Stop,
}
#[derive(Debug, Default)]
pub struct Sequence {
    pub track: Option<Track>,
    pub muted: bool,
    pub theme_finished: bool,
    pub flight_ready: bool,
    in_intro: bool,
    view: usize,
    revision: u64,
}
impl Sequence {
    fn select(&mut self, track: Option<Track>, restart: bool) {
        if self.track != track || restart {
            self.track = track;
            self.revision += 1;
        }
    }
    pub fn event(&mut self, event: Event) {
        match event {
            Event::BeginIntro => {
                self.in_intro = true;
                self.flight_ready = false;
                self.theme_finished = false;
                self.select(None, true);
            }
            Event::StartTheme if self.in_intro => self.select(Some(Track::Theme), true),
            Event::SkipIntro => {
                self.in_intro = false;
                self.view = 0;
                self.select(Some(Track::Tie), true);
            }
            Event::FinishIntro => {
                self.in_intro = false;
                self.view = 0;
                self.select(Some(Track::Cantina), false);
            }
            Event::View(view) if !self.in_intro => {
                self.view = view;
                self.flight_ready = false;
                let track = match view {
                    8 => Track::Takeoff,
                    7 => Track::Blaster,
                    6 => Track::Saber,
                    5 => Track::Droid,
                    1 => Track::Falcon,
                    _ => Track::Cantina,
                };
                self.select(Some(track), track != Track::Cantina);
            }
            Event::Boost if !self.in_intro && self.view == 8 && self.flight_ready => {
                self.select(Some(Track::Boost), true)
            }
            Event::Fire if !self.in_intro && self.view == 7 => self.select(Some(Track::Shot), true),
            Event::Finished(track) if self.track == Some(track) => match track {
                Track::Takeoff if !self.in_intro && self.view == 8 => {
                    self.flight_ready = true;
                    self.select(Some(Track::FlightAmbient), false);
                }
                Track::Boost if !self.in_intro && self.view == 8 => {
                    self.select(Some(Track::FlightAmbient), false)
                }
                Track::Theme => {
                    self.theme_finished = true;
                    self.select(None, false);
                }
                Track::Saber if !self.in_intro && self.view == 6 => {
                    self.select(Some(Track::Vader), false)
                }
                Track::Tie | Track::Droid | Track::Falcon | Track::Blaster | Track::Shot
                    if !self.in_intro =>
                {
                    self.select(Some(Track::Cantina), false)
                }
                _ => {}
            },
            Event::ToggleMute => self.muted = !self.muted,
            Event::Stop => self.select(None, true),
            _ => {}
        }
    }
}
#[derive(Clone, Debug)]
pub struct Status {
    pub track: Option<Track>,
    pub position: f32,
    pub durations: [f32; 12],
    pub theme_finished: bool,
    pub flight_ready: bool,
    pub muted: bool,
    pub available: bool,
}
impl Default for Status {
    fn default() -> Self {
        Self {
            track: None,
            position: 0.,
            durations: [
                90.112,
                90.112,
                54.6133,
                4.096,
                13.6533,
                3.4133,
                8.192,
                2.048,
                0.682667,
                crate::flight::TAKEOFF_SECONDS,
                97.621_33,
                crate::flight::BOOST_SECONDS,
            ],
            theme_finished: false,
            flight_ready: false,
            muted: false,
            available: false,
        }
    }
}
enum Command {
    Event(Event),
    SeekNearEnd,
    Shutdown,
}
pub struct Audio {
    sender: mpsc::Sender<Command>,
    status: Arc<Mutex<Status>>,
    worker: Option<thread::JoinHandle<()>>,
}
impl Default for Audio {
    fn default() -> Self {
        Self::new()
    }
}
impl Audio {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        let status = Arc::new(Mutex::new(Status::default()));
        let shared = status.clone();
        let (ready, started) = mpsc::channel();
        let worker = thread::spawn(move || {
            let device = RaylibAudio::init_audio_device().ok();
            let music: Vec<_> = Track::ALL
                .iter()
                .map(|&track| {
                    let path = format!("assets/audio/{}", track.filename());
                    let mut loaded = device.as_ref().and_then(|d| {
                        d.new_music(&path)
                            .map_err(|e| eprintln!("Audio {path}: {e}"))
                            .ok()
                    });
                    if let Some(m) = &mut loaded {
                        m.set_looping(track.looping());
                    }
                    loaded
                })
                .collect();
            let mut durations = Status::default().durations;
            for track in Track::ALL {
                if let Some(m) = &music[track as usize] {
                    durations[track as usize] = m.get_time_length();
                }
            }
            let available = music.iter().all(Option::is_some);
            if !available {
                eprintln!(
                    "Audio: faltan archivos o un dispositivo; se conserva la secuencia en silencio."
                );
            }
            {
                let mut s = shared.lock().unwrap();
                s.durations = durations;
                s.available = available;
            }
            let _ = ready.send(());
            let mut state = Sequence::default();
            let mut applied = u64::MAX;
            let mut playing = None;
            let mut began = Instant::now();
            'audio: loop {
                // Sólo este hilo toca los streams. Cada cambio detiene el anterior
                // antes de iniciar el siguiente, incluso con pulsaciones rápidas.
                match receiver.recv_timeout(Duration::from_millis(5)) {
                    Ok(command) => {
                        let mut commands = vec![command];
                        commands.extend(receiver.try_iter());
                        for command in commands {
                            match command {
                                Command::Shutdown => break 'audio,
                                Command::Event(event) => state.event(event),
                                Command::SeekNearEnd => {
                                    if let Some(track) = playing
                                        && let Some(m) = &music[track as usize]
                                    {
                                        m.seek_stream((durations[track as usize] - 0.35).max(0.));
                                    }
                                }
                            }
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                }
                if applied != state.revision {
                    if let Some(track) = playing
                        && let Some(m) = &music[track as usize]
                    {
                        m.stop_stream();
                    }
                    playing = state.track;
                    began = Instant::now();
                    applied = state.revision;
                    if let Some(track) = playing
                        && let Some(m) = &music[track as usize]
                    {
                        m.set_volume(if state.muted { 0. } else { track.volume() });
                        m.play_stream();
                    }
                    eprintln!("Audio: {}", playing.map_or("silencio", Track::filename));
                }
                let mut position = 0.;
                if let Some(track) = playing {
                    let index = track as usize;
                    if let Some(m) = &music[index] {
                        m.set_volume(if state.muted { 0. } else { track.volume() });
                        m.update_stream();
                        position = m.get_time_played();
                        if !m.is_stream_playing() && began.elapsed() > Duration::from_millis(100) {
                            state.event(Event::Finished(track));
                        }
                    } else {
                        position = began.elapsed().as_secs_f32();
                        if !track.looping() && position >= durations[index] {
                            state.event(Event::Finished(track));
                        }
                    }
                }
                if applied != state.revision {
                    continue;
                }
                *shared.lock().unwrap() = Status {
                    track: state.track,
                    position,
                    durations,
                    theme_finished: state.theme_finished,
                    flight_ready: state.flight_ready,
                    muted: state.muted,
                    available,
                };
            }
            if let Some(track) = playing
                && let Some(m) = &music[track as usize]
            {
                m.stop_stream();
            }
            // Music se destruye antes del dispositivo que lo creó.
        });
        let _ = started.recv_timeout(Duration::from_secs(5));
        Self {
            sender,
            status,
            worker: Some(worker),
        }
    }
    pub fn event(&self, event: Event) {
        let _ = self.sender.send(Command::Event(event));
    }
    pub fn status(&self) -> Status {
        self.status.lock().unwrap().clone()
    }
    pub fn theme_duration(&self) -> f32 {
        self.status().durations[Track::Theme as usize]
    }
}
impl Drop for Audio {
    fn drop(&mut self) {
        let _ = self.sender.send(Command::Shutdown);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// Verifica el decodificador real y los finales de los streams, en silencio.
/// Adelanta únicamente las pruebas; la reproducción normal respeta los archivos completos.
pub fn check() -> Result<(), Box<dyn std::error::Error>> {
    let audio = Audio::new();
    if !audio.status().available {
        return Err("No se cargaron los doce archivos de audio".into());
    }
    for track in Track::ALL {
        println!(
            "{}: {:.3}s, bucle={}",
            track.filename(),
            audio.status().durations[track as usize],
            track.looping()
        );
    }
    audio.event(Event::ToggleMute);
    let wait = |expected: Option<Track>| -> Result<(), Box<dyn std::error::Error>> {
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(3) {
            if audio.status().track == expected {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(10));
        }
        Err(format!("Audio esperado {expected:?}, recibido {:?}", audio.status()).into())
    };
    audio.event(Event::BeginIntro);
    audio.event(Event::StartTheme);
    wait(Some(Track::Theme))?;
    audio.sender.send(Command::SeekNearEnd)?;
    wait(None)?;
    if !audio.status().theme_finished {
        return Err("El final del tema no se detectó".into());
    }
    audio.event(Event::FinishIntro);
    wait(Some(Track::Cantina))?;
    for (event, effect, next) in [
        (Event::SkipIntro, Track::Tie, Track::Cantina),
        (Event::View(6), Track::Saber, Track::Vader),
        (Event::View(5), Track::Droid, Track::Cantina),
        (Event::View(1), Track::Falcon, Track::Cantina),
        (Event::View(7), Track::Blaster, Track::Cantina),
        (Event::Fire, Track::Shot, Track::Cantina),
    ] {
        audio.event(event);
        wait(Some(effect))?;
        audio.sender.send(Command::SeekNearEnd)?;
        wait(Some(next))?;
    }
    // Comprobar que ambas músicas continúan después de cruzar su final.
    for (view, track) in [(0, Track::Cantina), (6, Track::Vader)] {
        audio.event(Event::View(view));
        if view == 6 {
            wait(Some(Track::Saber))?;
            audio.sender.send(Command::SeekNearEnd)?;
        }
        wait(Some(track))?;
        audio.sender.send(Command::SeekNearEnd)?;
        thread::sleep(Duration::from_millis(800));
        let status = audio.status();
        if status.track != Some(track) || !(0.1..1.5).contains(&status.position) {
            return Err(format!("La música no volvió al inicio del bucle: {status:?}").into());
        }
    }
    for (view, effect) in [
        (6, Track::Saber),
        (5, Track::Droid),
        (1, Track::Falcon),
        (7, Track::Blaster),
    ] {
        audio.event(Event::View(view));
        wait(Some(effect))?;
        thread::sleep(Duration::from_millis(250));
        audio.event(Event::View(view));
        thread::sleep(Duration::from_millis(50));
        let status = audio.status();
        if status.track != Some(effect) || status.position > 0.2 {
            return Err(format!("El efecto no se reinició: {status:?}").into());
        }
    }
    for _ in 0..2 {
        audio.event(Event::Fire);
        wait(Some(Track::Shot))?;
        thread::sleep(Duration::from_millis(250));
        audio.event(Event::Fire);
        thread::sleep(Duration::from_millis(50));
        if audio.status().position > 0.2 {
            return Err("No se reinició el disparo manual".into());
        }
    }
    audio.event(Event::View(8));
    wait(Some(Track::Takeoff))?;
    audio.event(Event::Boost);
    wait(Some(Track::Takeoff))?;
    audio.sender.send(Command::SeekNearEnd)?;
    wait(Some(Track::FlightAmbient))?;
    audio.sender.send(Command::SeekNearEnd)?;
    thread::sleep(Duration::from_millis(800));
    if audio.status().track != Some(Track::FlightAmbient) || audio.status().position > 1.5 {
        return Err("La música espacial no volvió al inicio".into());
    }
    audio.event(Event::Boost);
    wait(Some(Track::Boost))?;
    audio.sender.send(Command::SeekNearEnd)?;
    wait(Some(Track::FlightAmbient))?;
    audio.event(Event::View(0));
    wait(Some(Track::Cantina))?;
    audio.event(Event::BeginIntro);
    wait(None)?;
    println!("Audio: doce archivos, secuencias y bucles verificados sin superposición.");
    Ok(())
}
