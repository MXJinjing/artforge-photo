#!/usr/bin/env python3
from pathlib import Path
import tempfile
import unittest

from check_clipboard_signing import clipboard_allowed, configured_profile


class ClipboardSigningTests(unittest.TestCase):
    def test_profile_acl_and_system_levels(self):
        self.assertFalse(clipboard_allowed({}))
        self.assertFalse(
            clipboard_allowed(
                {"bundle-info": {"apl": "normal"}, "acls": {"allowed-acls": []}}
            )
        )
        self.assertTrue(
            clipboard_allowed(
                {
                    "bundle-info": {"apl": "normal"},
                    "acls": {
                        "allowed-acls": ["ohos.permission.READ_PASTEBOARD"]
                    },
                }
            )
        )
        self.assertTrue(
            clipboard_allowed({"bundle-info": {"apl": "system_basic"}})
        )

    def test_unsigned_project_has_no_profile(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "build-profile.json5").write_text('{"app": {}}')
            self.assertIsNone(configured_profile(root))


if __name__ == "__main__":
    unittest.main()
