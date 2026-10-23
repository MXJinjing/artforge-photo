#!/usr/bin/env python3
"""Exercise JSON5 parsing and signing configuration resolution."""
import importlib.util
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location(
    'build_profile', Path(__file__).with_name('build_profile.py')
)
build_profile = importlib.util.module_from_spec(spec)
spec.loader.exec_module(build_profile)


class BuildProfileTests(unittest.TestCase):
    def test_json5_comments_and_trailing_commas(self):
        text = """{
            // the release product uses the release signing
            "app": {
                "signingConfigs": [
                    {"name": "release", "type": "HarmonyOS",},
                ],
                "products": [{"name": "default", "signingConfig": "release"},],
            },
        }"""
        profile = build_profile.parse_profile(text)
        self.assertIsNotNone(profile)
        self.assertEqual(
            build_profile.product_signing_name(profile), 'release'
        )
        self.assertEqual(
            build_profile.signing_config(profile, 'release')['type'], 'HarmonyOS'
        )

    def test_invalid_profile_is_rejected(self):
        self.assertIsNone(build_profile.parse_profile('{"app": '))

    def test_missing_profile_file(self):
        with tempfile.TemporaryDirectory() as directory:
            self.assertIsNone(
                build_profile.load_profile(Path(directory) / 'build-profile.json5')
            )

    def test_active_config_follows_the_product(self):
        profile = build_profile.parse_profile(
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
        config = build_profile.active_signing_config(profile)
        self.assertEqual(config['name'], 'debug')
        self.assertEqual(
            build_profile.material_path(config, 'profile'), Path('/tmp/debug.p7b')
        )

    def test_relative_material_path_resolves_against_the_root(self):
        config = {'material': {'profile': 'signing/release.p7b'}}
        self.assertEqual(
            build_profile.material_path(config, 'profile', '/tmp/project'),
            Path('/tmp/project/signing/release.p7b'),
        )
        self.assertIsNone(build_profile.material_path(config, 'certpath', '/tmp'))

    def test_profile_certificates_extracts_der(self):
        text = (
            '-----BEGIN CERTIFICATE-----\n'
            'MIIB\n'
            '-----END CERTIFICATE-----\n'
            '-----BEGIN CERTIFICATE-----\n'
            'AAEC\n'
            '-----END CERTIFICATE-----\n'
        )
        self.assertEqual(
            build_profile.profile_certificates(text),
            [b'\x30\x82\x01', b'\x00\x01\x02'],
        )
        self.assertEqual(build_profile.profile_certificates('no certificates'), [])


if __name__ == '__main__':
    unittest.main()
