#!/usr/bin/env python3
"""Check the restricted clipboard ACL in the active signing Profile."""

import argparse
import json
from pathlib import Path
import re
import subprocess

from build_profile import (
    BUILD_PROFILE,
    load_profile,
    material_path,
    product_signing_name,
    signing_config,
)

PERMISSION = "ohos.permission.READ_PASTEBOARD"


def clipboard_allowed(profile):
    level = profile.get("bundle-info", {}).get("apl", "normal")
    allowed = profile.get("acls", {}).get("allowed-acls", [])
    return level in ("system_basic", "system_core") or PERMISSION in allowed


def configured_profile(root):
    """Path of the Profile used by the build.

    Several signing configs may be declared, for example the automatic debug
    signing and a manually issued release signing. The one used by the build is
    selected through ``app.products[].signingConfig``; a hand-written profile
    without that reference falls back to a textual scan.
    """
    root = Path(root)
    profile = load_profile(root / BUILD_PROFILE)
    if profile is not None:
        name = product_signing_name(profile)
        if name:
            config = signing_config(profile, name)
            if config is None:
                raise ValueError(f"signing config '{name}' is not declared")
            return material_path(config, "profile", root)
    text = (root / BUILD_PROFILE).read_text()
    paths = list(
        dict.fromkeys(re.findall(r"[\"']?profile[\"']?\s*:\s*[\"']([^\"']+)", text))
    )
    if not paths:
        return None
    if len(paths) != 1:
        raise ValueError(
            "several local signing Profiles; select one with "
            "app.products[].signingConfig"
        )
    path = Path(paths[0]).expanduser()
    return path if path.is_absolute() else root / path


def read_profile(path):
    result = subprocess.run(
        [
            "openssl",
            "cms",
            "-verify",
            "-inform",
            "DER",
            "-in",
            str(path),
            "-noverify",
        ],
        capture_output=True,
        check=True,
        timeout=15,
    )
    return json.loads(result.stdout)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parent.parent,
    )
    args = parser.parse_args()
    root = args.root.resolve()
    manifest = (root / "entry/src/main/module.json5").read_text()
    if not re.search(
        r"\"name\"\s*:\s*\"" + re.escape(PERMISSION) + r"\"",
        manifest,
    ):
        print("Clipboard signing: READ_PASTEBOARD is not declared.")
        return 0
    try:
        profile_path = configured_profile(root)
    except ValueError as error:
        print(f"Clipboard signing: cannot select active Profile: {error}.")
        return 1
    if profile_path is None:
        print(
            "Clipboard signing: configure a local Profile containing the "
            "READ_PASTEBOARD ACL before device installation."
        )
        return 1
    try:
        profile = read_profile(profile_path)
    except (OSError, subprocess.SubprocessError, ValueError):
        print("Clipboard signing: unable to read the configured Profile.")
        return 1
    app = json.loads((root / "AppScope/app.json5").read_text())["app"]
    bundle = profile.get("bundle-info", {}).get("bundle-name")
    if bundle != app["bundleName"]:
        print(
            "Clipboard signing: Profile bundle does not match "
            f"{app['bundleName']}. Generate a new Profile for this app."
        )
        return 1
    if not clipboard_allowed(profile):
        print(
            "Clipboard signing: Profile lacks READ_PASTEBOARD ACL; "
            "installation would fail with 9568289."
        )
        return 1
    print(
        "Clipboard signing: Profile permits READ_PASTEBOARD. "
        "Certificate trust, expiry and device eligibility are not checked."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
