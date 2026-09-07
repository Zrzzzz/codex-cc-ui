#!/usr/bin/env python3
"""Package a prebuilt Linux CCUI binary as a standalone archive and npm tarball."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile


def main() -> None:
    root = Path(__file__).resolve().parent.parent
    os.environ.setdefault("CODEX_REPO_ROOT", str(root))
    sys.path.insert(0, str(root / "scripts"))
    from codex_package.cli import parse_package_version

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", required=True, type=parse_package_version)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--upstream-package", required=True, type=Path)
    parser.add_argument("--output-dir", required=True, type=Path)
    parser.add_argument("--profile", required=True, choices=("dev", "release"))
    args = parser.parse_args()
    output = args.output_dir.resolve()
    upstream = args.upstream_package.resolve()
    output.mkdir(parents=True, exist_ok=False)
    source_commit = subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=root, text=True
    ).strip()
    upstream_metadata = json.loads((upstream / "codex-package.json").read_text())
    if upstream_metadata["target"] != "x86_64-unknown-linux-musl":
        parser.error("The companion package must be for Linux x86_64 musl")

    target = "x86_64-unknown-linux-gnu"
    package_name = f"codex-cc-ui-{args.version}-{target}"
    package = output / package_name
    subprocess.run(
        [
            sys.executable,
            str(root / "scripts" / "build_codex_package.py"),
            "--target",
            target,
            "--package-version",
            args.version,
            "--package-dir",
            str(package),
            "--entrypoint-bin",
            str(args.binary.resolve()),
            "--code-mode-host-bin",
            str(upstream / "bin" / "codex-code-mode-host"),
            "--bwrap-bin",
            str(upstream / "codex-resources" / "bwrap"),
            "--zsh-bin",
            str(upstream / "codex-resources" / "zsh" / "bin" / "zsh"),
            "--rg-bin",
            str(upstream / "codex-path" / "rg"),
        ],
        env={**os.environ, "CODEX_REPO_ROOT": str(root)},
        check=True,
    )
    subprocess.run(
        ["strip", "--strip-unneeded", str(package / "bin" / "codex")], check=True
    )
    (package / "codex-cc").symlink_to("bin/codex")
    for filename in ("LICENSE", "NOTICE"):
        shutil.copy2(root / filename, package / filename)
    shutil.copy2(root / "scripts" / "ccui-release" / "README.md", package / "README.md")

    files = {}
    for path in sorted(package.rglob("*")):
        if path.is_file() and not path.is_symlink():
            with path.open("rb") as stream:
                files[path.relative_to(package).as_posix()] = hashlib.file_digest(
                    stream, "sha256"
                ).hexdigest()
    provenance = {
        "project": "codex-claude-code-ui",
        "version": args.version,
        "sourceCommit": source_commit,
        "target": target,
        "profile": args.profile,
        "minimumGlibc": "2.39",
        "requires": ["libssl.so.3", "libcrypto.so.3", "libgcc_s.so.1"],
        "companionPackage": upstream_metadata,
        "sha256": files,
    }
    (package / "BUILD-INFO.json").write_text(json.dumps(provenance, indent=2) + "\n")
    archive = output / f"{package_name}.tar.gz"
    with tarfile.open(archive, "w:gz") as tar:
        tar.add(package, arcname=package_name)

    npm_dir = output / "npm"
    npm_dir.mkdir()
    shutil.copytree(package, npm_dir / "native", symlinks=True)
    for filename in ("README.md", "LICENSE", "NOTICE"):
        shutil.copy2(package / filename, npm_dir / filename)
    manifest = {
        "name": "codex-cc-ui",
        "version": args.version,
        "description": "codex-claude-code-ui: a compact native Codex terminal interface",
        "license": "Apache-2.0",
        "author": "Zrzzzz <1160026659@qq.com>",
        "repository": {
            "type": "git",
            "url": "git+https://github.com/Zrzzzz/codex-cc-ui.git",
        },
        "homepage": "https://github.com/Zrzzzz/codex-cc-ui#readme",
        "bin": {"codex-cc": "native/bin/codex"},
        "os": ["linux"],
        "cpu": ["x64"],
        "libc": ["glibc"],
        "files": ["native", "README.md", "LICENSE", "NOTICE"],
    }
    (npm_dir / "package.json").write_text(json.dumps(manifest, indent=2) + "\n")
    result = subprocess.check_output(
        [
            "npm",
            "pack",
            "--json",
            "--ignore-scripts",
            "--pack-destination",
            str(output),
        ],
        cwd=npm_dir,
        text=True,
    )
    npm_archive = output / json.loads(result)[0]["filename"]
    with (output / "SHA256SUMS").open("w") as checksums:
        for path in (archive, npm_archive):
            with path.open("rb") as stream:
                digest = hashlib.file_digest(stream, "sha256").hexdigest()
            checksums.write(f"{digest}  {path.name}\n")
    print(output)


if __name__ == "__main__":
    main()
