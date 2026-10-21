#!/usr/bin/env python3
"""Validate the actual file associations and catch broken matching metadata."""
import copy
import unittest
from pathlib import Path

from json5_config import load as load_json5
from verify_native import verify_file_open

ROOT = Path(__file__).resolve().parents[1]


class FileOpenTests(unittest.TestCase):
    def setUp(self):
        self.config = load_json5(ROOT / 'entry/src/main/module.json5')
        self.config['app'] = load_json5(ROOT / 'AppScope/app.json5')['app']
        self.utds = load_json5(ROOT / 'entry/src/main/resources/rawfile/arkdata/utd/utd.json5')

    def test_actual_declarations(self):
        verify_file_open(self.config, self.utds)

    def test_sharing_metadata_cannot_replace_open_matching(self):
        for field in ('type', 'linkFeature'):
            with self.subTest(field=field):
                config = copy.deepcopy(self.config)
                del config['module']['abilities'][0]['skills'][1]['uris'][0][field]
                with self.assertRaises(ValueError):
                    verify_file_open(config, self.utds)

    def test_custom_namespace_must_match_bundle(self):
        self.utds['UniformDataTypeDeclarations'][0]['TypeId'] = 'wrong.pcraft'
        with self.assertRaises(ValueError):
            verify_file_open(self.config, self.utds)

    def test_missing_custom_registration(self):
        self.utds['UniformDataTypeDeclarations'].pop(0)
        with self.assertRaises(ValueError):
            verify_file_open(self.config, self.utds)


if __name__ == '__main__':
    unittest.main()
