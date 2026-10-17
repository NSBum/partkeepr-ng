#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Exercise version changes in temporary files, never the working checkout."""
import contextlib
import importlib.util
import io
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("release_version", Path(__file__).with_name("version.py"))
version = importlib.util.module_from_spec(spec)
spec.loader.exec_module(version)


class VersionTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.patch = patch.object(version, "ROOT", self.root)
        self.patch.start()
        self.addCleanup(self.patch.stop)
        (self.root / "backend").mkdir()
        (self.root / "Cargo.toml").write_text('[workspace.package]\nversion = "0.1.13"\n')
        (self.root / "backend/Cargo.toml").write_text('[package]\nname = "partkeepr-api"\nversion = "0.1.13"\n')
        (self.root / "Cargo.lock").write_text('version = 4\n\n[[package]]\nname = "other"\nversion = "0.1.13"\n\n[[package]]\nname = "partkeepr-api"\nversion = "0.1.13"\n')

    def run_cli(self, *args):
        with patch.object(sys, "argv", ["version.py", *args]), contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            return version.main()

    def test_component_bumps_update_only_the_application(self):
        for component, expected in [("patch", "0.1.14"), ("minor", "0.2.0"), ("major", "1.0.0")]:
            with self.subTest(component=component):
                # Reset these same fixtures between component cases.
                old = version.current_version()
                for path in [self.root / "Cargo.toml", self.root / "backend/Cargo.toml", self.root / "Cargo.lock"]:
                    text = path.read_text().replace('version = "' + old + '"', 'version = "0.1.13"')
                    path.write_text(text)
                self.assertEqual(self.run_cli("--bump", component), 0)
                self.assertEqual(version.current_version(), expected)
                self.assertIn('name = "other"\nversion = "0.1.13"', (self.root / "Cargo.lock").read_text())

    def test_mismatch_is_rejected_without_partial_writes(self):
        (self.root / "backend/Cargo.toml").write_text('[package]\nversion = "9.0.0"\n')
        before = (self.root / "Cargo.toml").read_text()
        self.assertEqual(self.run_cli("--set", "0.2.0"), 1)
        self.assertEqual((self.root / "Cargo.toml").read_text(), before)

    def test_requires_a_higher_numeric_version(self):
        for value in ["0.1.13", "0.1.12", "0.2.0-rc1", "01.2.0", "invalid"]:
            self.assertEqual(self.run_cli("--set", value), 1)
        self.assertEqual(version.current_version(), "0.1.13")

    def test_check_requires_current_dated_changelog(self):
        self.assertEqual(self.run_cli("--check"), 1)
        (self.root / "CHANGELOG.md").write_text('## [0.1.12] - 2026-10-17\n')
        self.assertEqual(self.run_cli("--check"), 1)
        (self.root / "CHANGELOG.md").write_text('## [0.1.13] - 2026-10-17\n')
        self.assertEqual(self.run_cli("--check"), 0)


if __name__ == "__main__":
    unittest.main()
