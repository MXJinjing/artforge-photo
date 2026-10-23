#!/usr/bin/env bash
# Build signed, ARM64 Release HAP and App Pack (.app) for AppGallery submission.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [[ -f "$ROOT/scripts/dev.local.env" ]]; then source "$ROOT/scripts/dev.local.env"; fi

usage() { echo "Usage: $0 [--version X.Y.Z[.REV]]"; }
VERSION=""
while (($#)); do
  case "$1" in
    --version) (($# >= 2)) || { usage >&2; exit 2; }; VERSION="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) usage >&2; exit 2 ;;
  esac
done
if [[ -z "$VERSION" ]]; then
  read -r -p 'Release version (e.g. 0.9.0.2): ' VERSION
fi
[[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]] || {
  echo 'Version must contain four numeric components, e.g. 0.9.0.2' >&2; exit 2;
}

VERSION_CODE="$(python3 - "$VERSION" <<'PY'
import sys
major, minor, patch, revision = map(int, sys.argv[1].split('.'))
if minor >= 100 or patch >= 100 or revision >= 100:
    raise SystemExit('Each minor, patch, and revision component must be below 100')
print(major * 1_000_000 + minor * 10_000 + patch * 100 + revision)
PY
)"

PROFILE="$ROOT/build-profile.json5"
APP_JSON="$ROOT/AppScope/app.json5"
[[ -f "$PROFILE" ]] || { echo 'Missing local build-profile.json5; configure signing in DevEco Studio first.' >&2; exit 1; }
# The release HAP must be signed with the signing config named "release".
# Several signing configs may coexist in build-profile.json5, for example the
# automatic debug signing and a manually issued release signing, so the config
# is selected by name instead of by the product reference.
PYTHONPATH="$ROOT/scripts" python3 - "$PROFILE" "$ROOT" <<'PY'
import json
import pathlib
import subprocess
import sys

from build_profile import (
    RELEASE_SIGNING,
    load_profile,
    material_path,
    profile_certificates,
    signing_config,
)

path = pathlib.Path(sys.argv[1])
root = pathlib.Path(sys.argv[2])
profile = load_profile(path)
if profile is None:
    raise SystemExit(f'Cannot read {path}.')
config = signing_config(profile, RELEASE_SIGNING)
if config is None:
    raise SystemExit(f"No signing config named '{RELEASE_SIGNING}' in {path.name}.")
if config.get('type') != 'HarmonyOS':
    raise SystemExit(f"The '{RELEASE_SIGNING}' signing config is not a HarmonyOS config.")
material = config.get('material') or {}
for key in ('certpath', 'profile', 'storeFile', 'storePassword', 'keyPassword', 'keyAlias'):
    if not material.get(key):
        raise SystemExit(f"The '{RELEASE_SIGNING}' signing config is missing {key}.")
for key in ('certpath', 'profile', 'storeFile'):
    if not material_path(config, key, root).is_file():
        raise SystemExit(f'Release signing file is missing: {material[key]}')

# The Profile binds the certificate and the bundle name; a mismatch is only
# detected by AppGallery when the package is uploaded, so reject it here.
try:
    result = subprocess.run(
        ['openssl', 'cms', '-verify', '-inform', 'DER',
         '-in', str(material_path(config, 'profile', root)), '-noverify'],
        capture_output=True,
        check=True,
    )
except (OSError, subprocess.SubprocessError):
    raise SystemExit('The release Profile cannot be read.')
provision = json.loads(result.stdout)
bundle = provision.get('bundle-info', {}).get('bundle-name')
app_bundle = json.loads((root / 'AppScope/app.json5').read_text())['app']['bundleName']
if bundle != app_bundle:
    raise SystemExit(f"The release Profile is bound to {bundle}, not {app_bundle}.")
if provision.get('type') != 'release':
    raise SystemExit('The release Profile is not a release Profile.')
embedded = provision.get('bundle-info', {}).get('distribution-certificate', '')
chain = profile_certificates(material_path(config, 'certpath', root).read_text())
if not embedded or not set(profile_certificates(embedded)) & set(chain):
    raise SystemExit('The release Profile and the release certificate do not match.')
PY

TMP_APP="$(mktemp)"
TMP_PROFILE="$(mktemp)"
SIGNCHECK="$(mktemp -d)"
cp "$APP_JSON" "$TMP_APP"
cp "$PROFILE" "$TMP_PROFILE"
restore_files() {
  cp "$TMP_APP" "$APP_JSON"; rm -f "$TMP_APP"
  cp "$TMP_PROFILE" "$PROFILE"; rm -f "$TMP_PROFILE"
  rm -rf "$SIGNCHECK"
}
trap restore_files EXIT INT TERM
python3 - "$APP_JSON" "$VERSION" "$VERSION_CODE" <<'PY'
import json, pathlib, sys
path = pathlib.Path(sys.argv[1])
data = json.loads(path.read_text())
data['app']['versionName'] = sys.argv[2]
data['app']['versionCode'] = int(sys.argv[3])
path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n')
PY
# Sign this build with the release signing config; the local file keeps the
# debug reference for the regular development loop.
PYTHONPATH="$ROOT/scripts" python3 - "$PROFILE" <<'PY'
import json
import pathlib
import sys

from build_profile import DEFAULT_PRODUCT, RELEASE_SIGNING, load_profile

path = pathlib.Path(sys.argv[1])
profile = load_profile(path)
products = (profile.get('app') or {}).get('products') or []
target = next((p for p in products if p.get('name') == DEFAULT_PRODUCT), None)
if target is None:
    raise SystemExit(f"No product named '{DEFAULT_PRODUCT}' to sign.")
target['signingConfig'] = RELEASE_SIGNING
path.write_text(json.dumps(profile, ensure_ascii=False, indent=2) + '\n')
PY

HVIGOR="${PHOTOCRAFT_HVIGOR:-/Applications/DevEco-Studio.app/Contents/tools/hvigor/bin/hvigorw}"
OHPM="${PHOTOCRAFT_OHPM:-/Applications/DevEco-Studio.app/Contents/tools/ohpm/bin/ohpm}"
[[ -x "$HVIGOR" ]] || { echo "hvigorw not found: $HVIGOR" >&2; exit 1; }
[[ -x "$OHPM" ]] || { echo "ohpm not found: $OHPM" >&2; exit 1; }

export DEVECO_SDK_HOME="${DEVECO_SDK_HOME:-/Applications/DevEco-Studio.app/Contents/sdk}"
export JAVA_HOME="${JAVA_HOME:-/Applications/DevEco-Studio.app/Contents/jbr/Contents/Home}"
export PATH="/Applications/DevEco-Studio.app/Contents/tools/node/bin:$PATH"
JAVA="$JAVA_HOME/bin/java"
if [[ ! -x "$JAVA" ]]; then JAVA="$(command -v java || true)"; fi
[[ -n "$JAVA" && -x "$JAVA" ]] || { echo 'java not found; set JAVA_HOME.' >&2; exit 1; }
SIGN_TOOL="${PHOTOCRAFT_SIGN_TOOL:-$DEVECO_SDK_HOME/default/openharmony/toolchains/lib/hap-sign-tool.jar}"
[[ -f "$SIGN_TOOL" ]] || { echo "hap-sign-tool not found: $SIGN_TOOL" >&2; exit 1; }

PHOTOCRAFT_RUST_PROFILE=release PHOTOCRAFT_HAP_BUILD_MODE=release "$ROOT/scripts/build_native.sh"
cd "$ROOT"
"$OHPM" install
# assembleApp builds the module HAP and packs it into the App Pack (.app) that
# AppGallery accepts. The App Pack carries the signing block, its HAPs are the
# unsigned module artifacts, so verify the pack instead of only the HAP.
"$HVIGOR" assembleApp --mode project -p product=default -p buildMode=release --no-daemon

BUILT="$ROOT/entry/build/default/outputs/default/entry-default-signed.hap"
[[ -f "$BUILT" ]] || { echo 'Signed HAP was not produced; check the DevEco signing configuration.' >&2; exit 1; }
shopt -s nullglob
APPS=("$ROOT"/build/outputs/default/*-signed.app)
shopt -u nullglob
((${#APPS[@]} == 1)) || {
  echo 'Expected exactly one signed App Pack in build/outputs/default.' >&2; exit 1;
}
BUILT_APP="${APPS[0]}"

# Reject an App Pack that is signed with the debug certificate, whose embedded
# Profile is stale, or whose version does not match AppScope/app.json5.
if ! "$JAVA" -jar "$SIGN_TOOL" verify-app -inFile "$BUILT_APP" \
    -outCertChain "$SIGNCHECK/certs.cer" -outProfile "$SIGNCHECK/profile.p7b" \
    >"$SIGNCHECK/verify.log" 2>&1; then
  cat "$SIGNCHECK/verify.log" >&2
  echo 'The App Pack signature cannot be verified.' >&2
  exit 1
fi
PYTHONPATH="$ROOT/scripts" python3 - "$BUILT_APP" "$SIGNCHECK" "$PROFILE" "$APP_JSON" "$ROOT" <<'PY'
import json
import pathlib
import sys
import tempfile
import zipfile

from build_profile import (
    RELEASE_SIGNING,
    load_profile,
    material_path,
    profile_certificates,
    signing_config,
)
from verify_native import verify

app_pack, signcheck, profile_path, app_json, root = (
    pathlib.Path(value) for value in sys.argv[1:6]
)
config = signing_config(load_profile(profile_path), RELEASE_SIGNING)
certfile = material_path(config, 'certpath', root)
profile = material_path(config, 'profile', root)

if (signcheck / 'profile.p7b').read_bytes() != profile.read_bytes():
    raise SystemExit('The App Pack is not signed with the release Profile.')
chain = profile_certificates((signcheck / 'certs.cer').read_text())
if not set(chain) & set(profile_certificates(certfile.read_text())):
    raise SystemExit('The App Pack is not signed with the release certificate.')

app = json.loads(app_json.read_text())['app']
with zipfile.ZipFile(app_pack) as archive:
    packed = json.loads(archive.read('pack.info'))['summary']['app']
    if packed['bundleName'] != app['bundleName']:
        raise SystemExit(f"App Pack bundle mismatch: {packed['bundleName']}")
    version = packed['version']
    if version['name'] != app['versionName'] or version['code'] != app['versionCode']:
        raise SystemExit(f"App Pack version mismatch: {version}")
    haps = [name for name in archive.namelist() if name.endswith('.hap')]
    if len(haps) != 1:
        raise SystemExit(f'Expected exactly one HAP in the App Pack: {haps}')
    with tempfile.TemporaryDirectory() as tmp:
        hap = pathlib.Path(tmp, pathlib.Path(haps[0]).name)
        hap.write_bytes(archive.read(haps[0]))
        verify(hap, app['bundleName'])
print(f'App Pack verified: {app_pack.name}')
PY

OUTDIR="$ROOT/dist"
mkdir -p "$OUTDIR"
OUT="$OUTDIR/artforge-photo-${VERSION}.hap"
OUT_APP="$OUTDIR/artforge-photo-${VERSION}.app"
cp "$BUILT" "$OUT"
cp "$BUILT_APP" "$OUT_APP"
python3 "$ROOT/scripts/verify_native.py" "$OUT"
echo "Release HAP: $OUT"
echo "Release App: $OUT_APP"
echo "versionName=$VERSION versionCode=$VERSION_CODE"
