#!/usr/bin/env python3
"""Read the JSON5 build-profile.json5 and resolve its signing configurations.

DevEco Studio writes ``build-profile.json5`` as JSON5, so comments and trailing
commas are tolerated here. The helpers answer two questions:

* which signing config a product build would use, and
* where that config keeps its Profile, certificate and keystore.

Several signing configs may coexist, for example the automatic debug signing
and a manually issued release signing. The active one is selected through
``app.products[].signingConfig``.
"""

from __future__ import annotations

import base64
import json
from pathlib import Path
import re

BUILD_PROFILE = "build-profile.json5"
DEFAULT_PRODUCT = "default"
RELEASE_SIGNING = "release"


def _skip_ignored(text: str, index: int) -> int:
    """Skip whitespace and comments, returning the next significant index."""
    length = len(text)
    while index < length:
        char = text[index]
        if char.isspace():
            index += 1
            continue
        if text.startswith("//", index):
            end = text.find("\n", index)
            index = length if end < 0 else end + 1
            continue
        if text.startswith("/*", index):
            end = text.find("*/", index + 2)
            index = length if end < 0 else end + 2
            continue
        break
    return index


def json5_to_json(text: str) -> str:
    """Drop JSON5 comments and trailing commas, keeping string literals intact."""
    out: list[str] = []
    index = 0
    length = len(text)
    while index < length:
        char = text[index]
        if char == '"':
            end = index + 1
            while end < length:
                if text[end] == "\\":
                    end += 2
                    continue
                if text[end] == '"':
                    end += 1
                    break
                end += 1
            out.append(text[index:end])
            index = end
            continue
        if text.startswith("//", index) or text.startswith("/*", index):
            index = _skip_ignored(text, index)
            continue
        if char == ",":
            following = _skip_ignored(text, index + 1)
            if following < length and text[following] in "}]":
                index += 1
                continue
        out.append(char)
        index += 1
    return "".join(out)


def parse_profile(text: str) -> dict | None:
    """Parse build profile text; return ``None`` when it is invalid."""
    for candidate in (text, json5_to_json(text)):
        try:
            data = json.loads(candidate)
        except json.JSONDecodeError:
            continue
        if isinstance(data, dict):
            return data
    return None


def load_profile(path) -> dict | None:
    """Parse a build profile file; return ``None`` when it is missing or invalid."""
    profile_path = Path(path)
    if not profile_path.is_file():
        return None
    return parse_profile(profile_path.read_text())


def signing_configs(profile) -> dict:
    """Map signing config name to its config dict."""
    app = (profile or {}).get("app") or {}
    configs = app.get("signingConfigs") or []
    return {
        config["name"]: config
        for config in configs
        if isinstance(config, dict) and config.get("name")
    }


def product_signing_name(profile, product: str = DEFAULT_PRODUCT) -> str | None:
    """Name of the signing config referenced by the given product."""
    app = (profile or {}).get("app") or {}
    products = [entry for entry in (app.get("products") or []) if isinstance(entry, dict)]
    for entry in products:
        if entry.get("name") == product and entry.get("signingConfig"):
            return entry["signingConfig"]
    for entry in products:
        if entry.get("signingConfig"):
            return entry["signingConfig"]
    return None


def signing_config(profile, name: str) -> dict | None:
    """The signing config declared under the given name."""
    return signing_configs(profile).get(name)


def active_signing_config(profile, product: str = DEFAULT_PRODUCT) -> dict | None:
    """The signing config a product build would use."""
    name = product_signing_name(profile, product)
    return signing_config(profile, name) if name else None


def material_path(config, key: str, root=None) -> Path | None:
    """Resolve one material path of a signing config."""
    material = (config or {}).get("material") or {}
    value = material.get(key)
    if not value:
        return None
    path = Path(str(value)).expanduser()
    if not path.is_absolute():
        path = Path(root or ".") / path
    return path


def profile_certificates(text: str) -> list[bytes]:
    """DER bytes of every PEM certificate in the given text."""
    bodies = re.findall(
        r"-----BEGIN CERTIFICATE-----(.*?)-----END CERTIFICATE-----", text, re.S
    )
    return [base64.b64decode("".join(body.split())) for body in bodies]
