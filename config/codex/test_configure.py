import base64
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import os
import stat
import subprocess
import sys
import tempfile
import threading
import unittest

SCRIPT = Path(__file__).parent / "src/.local/bin/codex-otel-configure"


class IngressHandler(BaseHTTPRequestHandler):
    paths = []
    authorization: str | None = None

    def do_POST(self):
        self.__class__.paths.append(self.path)
        self.rfile.read(int(self.headers.get("Content-Length", "0")))
        if self.headers.get("Authorization") != self.__class__.authorization:
            self.send_response(401)
        else:
            self.send_response(200)
        self.end_headers()

    def log_message(self, format, *args):
        pass


class ConfigureTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.home = Path(self.temp.name)
        self.secret = self.home / ".local/state/opencode/otel-basic-auth"
        self.secret.parent.mkdir(parents=True)
        self.write_secret("first-user:first-password")
        fake_bin = self.home / "bin"
        fake_bin.mkdir()
        codex = fake_bin / "codex"
        codex.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
        codex.chmod(0o755)
        self.env = os.environ.copy()
        self.env.update({
            "HOME": str(self.home),
            "PATH": f"{fake_bin}:{self.env.get('PATH', '')}",
        })

    def tearDown(self):
        self.temp.cleanup()

    def write_secret(self, value):
        encoded = base64.b64encode(value.encode()).decode()
        self.secret.write_text(encoded + "\n", encoding="utf-8")
        self.secret.chmod(0o600)
        return encoded

    def run_cli(self, *args):
        return subprocess.run(
            [sys.executable, str(SCRIPT), *args],
            env=self.env,
            text=True,
            capture_output=True,
            check=False,
        )

    def test_apply_preserves_existing_config_and_rotates_secret(self):
        config = self.home / ".codex/config.toml"
        config.parent.mkdir()
        config.write_text('[projects."/tmp/project"]\ntrust_level = "trusted"\n', encoding="utf-8")

        first = self.run_cli("apply")
        self.assertEqual(first.returncode, 0, first.stderr)
        text = config.read_text(encoding="utf-8")
        self.assertIn('[projects."/tmp/project"]', text)
        self.assertEqual(text.count("Authorization ="), 2)
        self.assertIn("Basic " + self.secret.read_text().strip(), text)
        self.assertEqual(stat.S_IMODE(config.stat().st_mode), 0o600)

        old = self.secret.read_text().strip()
        new = self.write_secret("second-user:second-password")
        second = self.run_cli("apply")
        self.assertEqual(second.returncode, 0, second.stderr)
        text = config.read_text(encoding="utf-8")
        self.assertNotIn(old, text)
        self.assertEqual(text.count(new), 2)
        self.assertEqual(text.count("devenv managed Codex telemetry >>>"), 1)

    def test_dry_run_redacts_secret_and_does_not_write(self):
        result = self.run_cli("apply", "--dry-run")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("[REDACTED]", result.stdout)
        self.assertNotIn(self.secret.read_text().strip(), result.stdout)
        self.assertFalse((self.home / ".codex/config.toml").exists())

    def test_refuses_insecure_secret_permissions(self):
        self.secret.chmod(0o644)
        result = self.run_cli("apply")
        self.assertEqual(result.returncode, 1)
        self.assertIn("group/world-accessible", result.stderr)
        self.assertFalse((self.home / ".codex/config.toml").exists())

    def test_refuses_unmanaged_otel_sections(self):
        config = self.home / ".codex/config.toml"
        config.parent.mkdir()
        config.write_text('[otel]\nenvironment = "other"\n', encoding="utf-8")
        result = self.run_cli("apply")
        self.assertEqual(result.returncode, 1)
        self.assertIn("refusing to overwrite unmanaged [otel]", result.stderr)

    def test_restores_existing_config_when_codex_rejects_candidate(self):
        config = self.home / ".codex/config.toml"
        config.parent.mkdir()
        original = '[projects."/tmp/project"]\ntrust_level = "trusted"\n'
        config.write_text(original, encoding="utf-8")
        config.chmod(0o640)
        encoded = self.secret.read_text().strip()
        codex = self.home / "bin/codex"
        codex.write_text(
            f"#!/bin/sh\necho 'rejected Basic {encoded}' >&2\nexit 1\n",
            encoding="utf-8",
        )
        codex.chmod(0o755)

        result = self.run_cli("apply")

        self.assertEqual(result.returncode, 1)
        self.assertIn("Codex rejected config.toml: rejected Basic [REDACTED]", result.stderr)
        self.assertNotIn(encoded, result.stderr)
        self.assertEqual(config.read_text(encoding="utf-8"), original)
        self.assertEqual(stat.S_IMODE(config.stat().st_mode), 0o640)

    def test_check_validates_both_authenticated_ingress_paths(self):
        IngressHandler.paths = []
        IngressHandler.authorization = "Basic " + self.secret.read_text().strip()
        server = ThreadingHTTPServer(("127.0.0.1", 0), IngressHandler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        endpoint = f"http://127.0.0.1:{server.server_port}"
        try:
            applied = self.run_cli("apply", "--endpoint", endpoint)
            self.assertEqual(applied.returncode, 0, applied.stderr)
            checked = self.run_cli("check", "--endpoint", endpoint)
            self.assertEqual(checked.returncode, 0, checked.stderr)
            self.assertEqual(IngressHandler.paths, ["/v1/metrics", "/v1/traces"])
        finally:
            server.shutdown()
            server.server_close()
            thread.join(timeout=5)

    def test_rejects_plaintext_non_loopback_endpoint(self):
        result = self.run_cli("apply", "--endpoint", "http://telemetry.example.test")
        self.assertEqual(result.returncode, 1)
        self.assertIn("must use HTTPS", result.stderr)
        self.assertFalse((self.home / ".codex/config.toml").exists())


if __name__ == "__main__":
    unittest.main()
