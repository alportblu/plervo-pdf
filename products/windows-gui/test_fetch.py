import hashlib
from pathlib import Path
import tempfile
import unittest
import warnings
import zipfile

from fetch import extract_msi


class ArchiveBoundaryTests(unittest.TestCase):
    def test_only_the_exact_pinned_file_is_written(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            archive = root / "input.zip"
            with zipfile.ZipFile(archive, "w") as out:
                out.writestr("pinned.msi", b"synthetic MSI bytes")
                out.writestr("../../unexpected.exe", b"never extracted")
            expected = hashlib.sha256(b"synthetic MSI bytes").hexdigest()
            extract_msi(archive, "pinned.msi", root / "candidate.msi", expected)
            self.assertEqual((root / "candidate.msi").read_bytes(), b"synthetic MSI bytes")
            self.assertEqual({p.name for p in root.iterdir()}, {"input.zip", "candidate.msi"})

    def test_wrong_hash_and_ambiguous_names_are_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            archive = root / "input.zip"
            with zipfile.ZipFile(archive, "w") as out:
                out.writestr("pinned.msi", b"changed")
            with self.assertRaises(AssertionError):
                extract_msi(archive, "pinned.msi", root / "wrong.msi", "0" * 64)
            with zipfile.ZipFile(archive, "w") as out:
                out.writestr("folder/pinned.msi", b"wrong location")
            with self.assertRaises(AssertionError):
                extract_msi(archive, "pinned.msi", root / "missing.msi", "0" * 64)

    def test_duplicate_and_symlink_members_are_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            archive = root / "input.zip"
            with warnings.catch_warnings():
                warnings.simplefilter("ignore", UserWarning)
                with zipfile.ZipFile(archive, "w") as out:
                    out.writestr("pinned.msi", b"first")
                    out.writestr("pinned.msi", b"second")
            with self.assertRaises(AssertionError):
                extract_msi(archive, "pinned.msi", root / "duplicate.msi", "0" * 64)
            member = zipfile.ZipInfo("pinned.msi")
            member.create_system = 3
            member.external_attr = 0o120777 << 16
            with zipfile.ZipFile(archive, "w") as out:
                out.writestr(member, b"elsewhere")
            with self.assertRaises(AssertionError):
                extract_msi(archive, "pinned.msi", root / "symlink.msi", "0" * 64)


if __name__ == "__main__":
    unittest.main()
