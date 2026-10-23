import hashlib
import json
from pathlib import Path
import tempfile
import unittest

from verify import verify


class DistributionNotices(unittest.TestCase):
    def test_changed_dependencies_missing_notices_and_paths_fail_closed(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root / "packaging/licenses").mkdir(parents=True)
            original = b"reviewed\n"
            digest = hashlib.sha256(original).hexdigest()
            inputs = {"Cargo.lock": digest, "Cargo.toml": digest}
            manifest = {"inputs": inputs, "notices_lf_sha256": digest}
            path = root / "packaging/licenses/manifest.json"
            path.write_text(json.dumps(manifest))
            for name in [*inputs, "THIRD_PARTY_NOTICES.txt"]:
                (root / name).write_bytes(original)
            self.assertEqual(verify(root), 2)
            for name in [*inputs, "THIRD_PARTY_NOTICES.txt"]:
                with self.subTest(name=name):
                    (root / name).write_bytes(b"changed")
                    with self.assertRaises(ValueError):
                        verify(root)
                    (root / name).write_bytes(b"reviewed\r\n")
                    self.assertEqual(verify(root), 2)
            inputs["../outside"] = digest
            path.write_text(json.dumps(manifest))
            with self.assertRaises(ValueError):
                verify(root)
            del inputs["../outside"]
            path.write_text(json.dumps(manifest))
            (root / "THIRD_PARTY_NOTICES.txt").unlink()
            with self.assertRaises(OSError):
                verify(root)


if __name__ == "__main__":
    unittest.main()
