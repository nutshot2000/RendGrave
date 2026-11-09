"""Build a release and package it for itch.io.

    python tools/package.py

Makes dist/Rendgrave-EA-<version>-windows.zip (the game, its music, a player readme)
and dist/press/ (screenshots for the store page, if the --press tour has been run).
Run the game's press tour first for fresh screenshots:
    target/release/pixel_arpg.exe --tour press --press
    target/release/pixel_arpg.exe --spells press/spl --press
"""
import os
import re
import shutil
import subprocess
import zipfile

root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(root)
ver = re.search(r'^version\s*=\s*"([^"]+)"', open('Cargo.toml', encoding='utf8').read(), re.M).group(1)
name = f'Rendgrave-EA-{ver}-windows'

print('building release...')
# always build into target/release, whatever CARGO_TARGET_DIR the shell has set,
# so the exe that gets packaged is the one just built
env = {k: v for k, v in os.environ.items() if k != 'CARGO_TARGET_DIR'}
subprocess.run(['cargo', 'build', '--release'], check=True, env=env)

out = os.path.join('dist', name)
shutil.rmtree(out, ignore_errors=True)
os.makedirs(os.path.join(out, 'assets', 'music'))
shutil.copy2(os.path.join('target', 'release', 'pixel_arpg.exe'), os.path.join(out, 'Rendgrave.exe'))
for f in sorted(os.listdir(os.path.join('assets', 'music'))):
    if f.endswith('.wav'):
        shutil.copy2(os.path.join('assets', 'music', f), os.path.join(out, 'assets', 'music', f))
shutil.copy2(os.path.join('docs', 'README_PLAYERS.txt'), os.path.join(out, 'README.txt'))

zpath = os.path.join('dist', name + '.zip')
if os.path.exists(zpath):
    os.remove(zpath)
with zipfile.ZipFile(zpath, 'w', zipfile.ZIP_DEFLATED, compresslevel=9) as z:
    for dirpath, _, files in os.walk(out):
        for f in files:
            full = os.path.join(dirpath, f)
            z.write(full, os.path.relpath(full, 'dist'))
print(f'{zpath}  {os.path.getsize(zpath) / 1e6:.1f} MB')

# store-page screenshots
picks = [
    ('press/t11_keg_blast.png', '01_keg_blast.png'),
    ('press/spl/s02_meteor_impact.png', '02_meteor.png'),
    ('press/t04_fen.png', '03_fen.png'),
    ('press/t05_ravenhold.png', '04_ravenhold.png'),
    ('press/t09_cellars2_boss.png', '05_chief.png'),
    ('press/spl/s11_tab_storm.png', '06_skills.png'),
    ('press/t15_shop.png', '07_merchant.png'),
    ('press/spl/s07_storm.png', '08_storm.png'),
    ('menus/m1_title.png', '00_title.png'),
]
pdir = os.path.join('dist', 'press')
os.makedirs(pdir, exist_ok=True)
n = 0
for src, dst in picks:
    if os.path.exists(src):
        shutil.copy2(src, os.path.join(pdir, dst))
        n += 1
shutil.copy2(os.path.join('docs', 'itch_page.md'), os.path.join(pdir, 'itch_page.md'))
print(f'{n} screenshots and the page text in {pdir}')
