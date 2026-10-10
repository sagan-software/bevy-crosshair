"""Prove that the pinned Dylint library rejects a violation and accepts its fix."""

from pathlib import Path
import os
import subprocess
import sys
import tempfile


def main() -> None:
    """Run the external fixture without placing lint state in this repository."""
    with tempfile.TemporaryDirectory(prefix="crosshair-lint-probe-") as directory:
        root = Path(directory)
        (root / "src").mkdir()
        (root / "Cargo.toml").write_text(
            '[package]\nname = "lint-probe"\nversion = "0.1.0"\nedition = "2024"\n'
        )
        source = root / "src/lib.rs"
        source.write_text("pub fn count(values: Vec<u8>) -> usize { values.len() }\n")
        command = [
            "cargo", "dylint", "--no-metadata", "--no-build", "--lib-path", sys.argv[1],
            "--", "--lib",
        ]
        environment = os.environ | {"DYLINT_RUSTFLAGS": "-D ownership_at_boundaries"}
        failing = subprocess.run(command, cwd=root, env=environment, capture_output=True, text=True, check=False)
        transcript = failing.stdout + failing.stderr
        if failing.returncode == 0 or "ownership-at-boundaries" not in transcript.replace("_", "-"):
            raise RuntimeError(f"The lint probe did not reject the ownership violation:\n{transcript}")
        source.write_text("pub fn count(values: &[u8]) -> usize { values.len() }\n")
        fixed = subprocess.run(command, cwd=root, env=environment, capture_output=True, text=True, check=False)
        if fixed.returncode:
            raise RuntimeError(f"The corrected probe failed:\n{fixed.stdout}{fixed.stderr}")
        print("Dylint probe passed: ownership violation rejected, borrowed input accepted.")


if __name__ == "__main__":
    main()
