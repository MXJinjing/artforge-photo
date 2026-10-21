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

    def test_product_selects_its_signing_config(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "build-profile.json5").write_text(
                """
                {
                  "app": {
                    "signingConfigs": [
                      {"name": "debug", "material": {"profile": "/tmp/debug.p7b"}},
                      {"name": "release", "material": {"profile": "/tmp/release.p7b"}}
                    ],
                    "products": [{"name": "default", "signingConfig": "debug"}]
                  }
                }
                """
            )
            self.assertEqual(configured_profile(root), Path('/tmp/debug.p7b'))

    def test_release_config_is_used_when_the_product_points_at_it(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "build-profile.json5").write_text(
                """
                {
                  "app": {
                    "signingConfigs": [
                      {"name": "debug", "material": {"profile": "/tmp/debug.p7b"}},
                      {"name": "release", "material": {"profile": "/tmp/release.p7b"}}
                    ],
                    "products": [{"name": "default", "signingConfig": "release"}]
                  }
                }
                """
            )
            self.assertEqual(configured_profile(root), Path('/tmp/release.p7b'))

    def test_several_profiles_without_a_product_reference(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "build-profile.json5").write_text(
                """
                {
                  "app": {
                    "signingConfigs": [
                      {"name": "debug", "material": {"profile": "/tmp/debug.p7b"}},
                      {"name": "release", "material": {"profile": "/tmp/release.p7b"}}
                    ]
                  }
                }
                """
            )
            with self.assertRaises(ValueError):
                configured_profile(root)

    def test_single_profile_without_a_product_reference(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "build-profile.json5").write_text(
                '{"app": {"signingConfigs": ['
                '{"name": "default", "material": {"profile": "/tmp/only.p7b"}}]}}'
            )
            self.assertEqual(configured_profile(root), Path('/tmp/only.p7b'))


if __name__ == "__main__":
    unittest.main()
