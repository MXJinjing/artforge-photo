#!/usr/bin/env python3
"""Read the app's JSON5 configuration files with the standard library.

HarmonyOS manifests are `.json5`: comments and trailing commas are legal for the
DevEco build, while `json.loads` rejects them. Tests that inspect the real
manifest go through this helper so a comment cannot break them.
"""
import json
import pathlib


def strip_comments(text):
    """Drop `//` and `/* */` comments that are outside string literals."""
    out = []
    index = 0
    in_string = False
    escaped = False
    while index < len(text):
        char = text[index]
        if in_string:
            out.append(char)
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                in_string = False
            index += 1
            continue
        if char == '"':
            in_string = True
            out.append(char)
            index += 1
            continue
        if char == "/" and index + 1 < len(text) and text[index + 1] == "/":
            while index < len(text) and text[index] != "\n":
                index += 1
            continue
        if char == "/" and index + 1 < len(text) and text[index + 1] == "*":
            index += 2
            while index + 1 < len(text) and not (text[index] == "*" and text[index + 1] == "/"):
                index += 1
            index += 2
            continue
        out.append(char)
        index += 1
    return "".join(out)


def loads(text):
    """Parse JSON5 content that only uses comments and trailing commas."""
    without_comments = strip_comments(text)
    try:
        return json.loads(without_comments)
    except json.JSONDecodeError:
        # Trailing commas before a closing brace or bracket are the other JSON5
        # extension the manifests use.
        import re

        relaxed = re.sub(r",(\s*[}\]])", r"\1", without_comments)
        return json.loads(relaxed)


def load(path):
    return loads(pathlib.Path(path).read_text())
