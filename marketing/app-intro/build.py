"""Build the standalone HTML and an original, synthesized music bed."""
from pathlib import Path
import base64

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parent.parent
TEMP = Path('/var/folders/kw/xldbf_nn5z10vyjj7n5m40xm0000gn/T')
ASSETS = {
    'ICON': REPO / 'entry/src/main/resources/base/media/app_icon.png',
    'CURVES': TEMP / 'codex-clipboard-1603e184-7022-4e28-b47c-5a353f27bc74.jpg',
    'FOREST': TEMP / 'codex-clipboard-d8a3a78e-8622-4e33-8446-160dab13a26f.jpg',
    'HOME': TEMP / 'codex-clipboard-53f4474f-02ce-4601-b82c-75a5564c2b1b.jpg',
    'BRUSH': TEMP / 'codex-clipboard-261babda-09b1-4065-b578-373b480baca8.jpg',
    'LAKE': TEMP / 'codex-clipboard-6f3dd929-9d44-4411-80d6-45ef12d69ce0.jpg',
}
html = (ROOT / 'intro.template.html').read_text()
for key, path in ASSETS.items():
    mime = 'image/png' if path.suffix == '.png' else 'image/jpeg'
    html = html.replace('{{' + key + '}}', f'data:{mime};base64,' + base64.b64encode(path.read_bytes()).decode())
(ROOT / 'index.html').write_text(html)

# Use the original gentle cue for future rebuilds.
import runpy
runpy.run_path(str(ROOT / 'music.py'), run_name='__main__')
print('Built standalone HTML and original gentle score.')
