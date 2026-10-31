"""Render existing app UIs; retain original pixels and require Store-sized images."""
import argparse
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess


def main():
    root = Path(__file__).resolve().parents[3]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('product', choices=('photo', 'vector', 'develop', 'motion', 'publish', 'write', 'slides', 'sheets', 'audio', 'canvas'))
    parser.add_argument('source', type=Path)
    args = parser.parse_args()
    source = args.source.resolve(strict=True)
    output = root / 'dist/validation' / args.product / 'listing'
    output.mkdir(parents=True, exist_ok=True)
    png = output / 'en.png'
    script = output / 'capture.jsonl'
    script.write_text(json.dumps({'shot': str(png)}) + '\n')
    env = dict(os.environ, PDFCRAFT_SYSTEM_FONTS='0', FIGCRAFT_NO_SYSTEM_FONTS='1')
    env['PLERVO_' + args.product.upper() + '_CONFIG_DIR'] = str(output / 'isolated-settings')
    helper = None
    crate = {
        'photo': 'photocraft-ui-egui', 'vector': 'vectorcraft-ui-egui',
        'motion': 'effectcraft-ui-egui', 'publish': 'designcraft-ui-egui',
        'write': 'wordcraft-ui-egui', 'slides': 'deckcraft-ui-egui',
        'sheets': 'gridcraft-ui-egui', 'audio': 'soundcraft-ui-egui',
        'canvas': 'plervo-canvas', 'develop': 'plervo-develop-cli',
    }[args.product]
    extra = {
        'photo': ['--example', 'snapshot', '--', '--out', str(png), '--size', '1600x1000', '--scale', '1', '--safe-gpu', '--custom-titlebar'],
        'motion': ['--example', 'snapshot', '--', '--out', str(png), '--size', '1600x1000', '--scale', '1', '--settle', '1.5'],
        'publish': ['--example', 'ui_shot', '--', str(script)],
        'write': ['--example', 'ui_shot', '--', str(script)],
        'sheets': ['--example', 'snapshot', '--', '--sample', 'sales', '--size', '1600x1000', '--scale', '1', '--language', 'en', str(png)],
        'audio': ['--example', 'ui_shot', '--', str(png), '--size', '1600x1000'],
    }.get(args.product)
    if args.product in ('vector', 'slides', 'canvas'):
        folder = source / ('apps/figcraft/examples' if args.product == 'canvas' else 'crates/ui-egui/examples')
        folder.mkdir(parents=True, exist_ok=True)
        helper = folder / 'plervo_store_shot.rs'
        if helper.exists():
            raise ValueError('Refusing to overwrite an existing example')
        shutil.copyfile(Path(__file__).with_name('shot-' + args.product + '.rs'), helper)
        extra = ['--example', 'plervo_store_shot', '--', str(png)]
    if args.product == 'develop':
        script.write_text(json.dumps({'method': 'ui.settle', 'params': {'timeoutMs': 60000}}) + '\n')
        extra = ['--', 'snapshot', '--demo', '--script', str(script), '--size', '1600x1000', '-o', str(png)]
    try:
        if helper:
            subprocess.run(['cargo', 'clippy', '--locked', '-p', crate, '--example', 'plervo_store_shot', '--', '-D', 'warnings'], cwd=source, env=env, check=True, timeout=1200)
        subprocess.run(['cargo', 'run', '--locked', '-p', crate, *extra], cwd=source, env=env, check=True, timeout=1800)
        header = png.read_bytes()[:24]
        if header[:8] != b'\x89PNG\r\n\x1a\n':
            raise ValueError('Renderer did not write a PNG')
        width, height = struct.unpack('>II', header[16:24])
        if width < 1366 or height < 768:
            raise ValueError('Renderer output is below Store listing dimensions')
        (output / 'capture.json').write_text(json.dumps({'product': args.product, 'width': width, 'height': height, 'language': 'en', 'source': 'actual application UI; unscaled renderer output', 'visual_review_required': True}, indent=2) + '\n')
    finally:
        if helper:
            helper.unlink()


if __name__ == '__main__':
    main()
