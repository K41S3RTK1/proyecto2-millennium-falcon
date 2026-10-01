#!/usr/bin/env python3
"""Añade el segmento exportado por export_flight_video al video base, con WAV locales.
Uso: python3 scripts/compose_delivery_video.py base.mp4 vuelo-silent.mp4 salida.mp4
Requiere FFmpeg y ffprobe en PATH; sólo se usa para producir la entrega.
"""
import json
from pathlib import Path
import subprocess
import sys
import tempfile


def run(*args):
    subprocess.run(args, check=True)


def main():
    base, silent, output = map(lambda v: Path(v).resolve(), sys.argv[1:])
    if output in (base, silent):
        raise SystemExit('La salida debe ser distinta de las entradas')
    audio = Path(__file__).resolve().parents[1] / 'assets' / 'audio'
    files = ['Despegue.wav', 'ambientefalcon.wav', 'Impulso.wav', 'disparofalcon.wav', 'explotion.wav']
    filters = [
        '[0:a]volume=0.65,apad,atrim=duration=49.834667[takeoff]',
        '[1:a]asplit=2[amb1][amb2]',
        '[amb1]atrim=duration=4.165333,asetpts=PTS-STARTPTS,volume=0.45,adelay=49835:all=1[ambient1]',
        '[amb2]atrim=duration=6.202667,asetpts=PTS-STARTPTS,volume=0.45,adelay=73797:all=1[ambient2]',
        '[2:a]volume=0.65,adelay=54000:all=1[boost]',
        '[3:a]asplit=9' + ''.join(f'[shot{i}]' for i in range(9)),
        '[4:a]asplit=3[expl0][expl1][expl2]',
    ]
    names = ['takeoff', 'ambient1', 'ambient2', 'boost']
    for i, time in enumerate([55, 57, 59, 62, 64, 66, 69, 71, 73]):
        filters.append(f'[shot{i}]volume=0.65,adelay={time*1000}:all=1[s{i}]')
        names.append(f's{i}')
    for i, time in enumerate([59567, 66567, 73567]):
        filters.append(f'[expl{i}]volume=0.65,adelay={time}:all=1[e{i}]')
        names.append(f'e{i}')
    filters.append(''.join(f'[{n}]' for n in names) +
                   f'amix=inputs={len(names)}:normalize=0:duration=longest,alimiter=limit=0.95:level=false:latency=true,apad,atrim=duration=80[audio]')
    base_length = float(subprocess.check_output([
        'ffprobe', '-v', 'error', '-show_entries', 'format=duration', '-of', 'default=nw=1:nk=1', str(base)
    ]))
    with tempfile.TemporaryDirectory(prefix='falcon-entrega-') as work:
        work = Path(work)
        mixed, flight, listing, metadata = [work / n for n in ['mix.wav', 'vuelo.mp4', 'concat.txt', 'chapters.txt']]
        inputs = [v for name in files for v in ['-i', str(audio / name)]]
        run('ffmpeg', '-hide_banner', '-loglevel', 'warning', '-y', *inputs,
            '-filter_complex', ';'.join(filters), '-map', '[audio]', '-ar', '48000', '-ac', '2', '-c:a', 'pcm_s16le', str(mixed))
        run('ffmpeg', '-hide_banner', '-loglevel', 'warning', '-y', '-i', str(silent), '-i', str(mixed),
            '-map', '0:v:0', '-map', '1:a:0', '-c:v', 'copy', '-c:a', 'aac', '-b:a', '192k', '-t', '80', str(flight))
        # Escapar apóstrofes para el formato concat, no para un shell.
        listing.write_text(''.join("file '" + str(p).replace("'", "'\\''") + "'\n" for p in [base, flight]))
        chapters = [(0, 'Introduccion'), (94.112, 'Diorama: rotacion y materiales'),
                    (175.112, 'Soldados rebeldes'), (187.112, 'Skybox espacial'),
                    (base_length, 'Modo nave: despegue completo'),
                    (base_length+49.835, 'Tres cazas TIE'),
                    (base_length+54, 'Impulso y combate con audio'),
                    (base_length+73.567, 'Victoria')]
        lines = [';FFMETADATA1', 'title=Millennium Falcon - Diorama y combate espacial']
        for i, (start, title) in enumerate(chapters):
            end = chapters[i+1][0] if i+1 < len(chapters) else base_length+80
            lines += ['[CHAPTER]', 'TIMEBASE=1/1000', f'START={round(start*1000)}', f'END={round(end*1000)}', f'title={title}']
        metadata.write_text('\n'.join(lines)+'\n')
        run('ffmpeg', '-hide_banner', '-loglevel', 'warning', '-y', '-f', 'concat', '-safe', '0', '-i', str(listing),
            '-i', str(base), '-i', str(flight), '-i', str(metadata),
            '-filter_complex', f'[1:a]apad,atrim=duration={base_length},asetpts=PTS-STARTPTS[a0];[2:a]apad,atrim=duration=80,asetpts=PTS-STARTPTS[a1];[a0][a1]concat=n=2:v=0:a=1[fullaudio]',
            '-map', '0:v:0', '-map', '[fullaudio]', '-map_metadata', '3', '-map_chapters', '3',
            '-c:v', 'copy', '-c:a', 'aac', '-b:a', '192k', '-movflags', '+faststart', str(output))
    print(json.dumps({'video': str(output), 'duration_expected': base_length+80, 'shots': 9, 'explosions': 3}))


if __name__ == '__main__':
    main()
