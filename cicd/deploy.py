#!/usr/bin/env python3
from __future__ import annotations
import argparse
import os
import shutil
import subprocess
from pathlib import Path


def run(cmd: list[str], *, cwd: Path | None = None) -> None:
    """Pretty wrapper around subprocess.check_call."""
    print(f"▶ {' '.join(cmd)}")
    subprocess.check_call(cmd, cwd=cwd)


def main() -> None:
    home: str = os.getenv("HOME")
    parser = argparse.ArgumentParser(
        prog="deploy.py",
        description="Fresh-clone repo, drop release binary in place, write .env, restart compose.",
    )
    parser.add_argument(
        "--binary-path",
        required=True,
        type=Path,
        help="Absolute path to the compiled backend binary on the server",
    )
    parser.add_argument(
        "--repo-url",
        required=True,
        help="URL of the repository to clone",
    )
    parser.add_argument(
        "--api-key-chatgpt",
        required=True,
        help="ChatGPT API key to insert into .env",
    )
    parser.add_argument(
        "--deploy-dir",
        type=Path,
        default=Path(f"{home}/langample"),
        help="Root directory for the checkout on the server",
    )
    args = parser.parse_args()

    binary_src: Path = args.binary_path.resolve()
    if not binary_src.is_file():
        raise Exception(f"{binary_src} is not a file")
    api_key_chatgpt: str = args.api_key_chatgpt
    deploy_dir: Path = args.deploy_dir
    repo_url: str = args.repo_url

    bin_file_name = "backend-bin"
    compose_dir = deploy_dir / "docker"
    bin_dest = compose_dir / "langample" / bin_file_name
    env_file = compose_dir / ".env"

    # 1. Fresh repo
    print(f"▶ Cleaning {deploy_dir}")
    shutil.rmtree(deploy_dir, ignore_errors=True)

    print("▶ Cloning repo")
    run(["git", "clone", "--depth", "1", repo_url, str(deploy_dir)])

    # 2. move binary
    bin_dest.parent.mkdir(parents=True, exist_ok=True)
    print("▶ Moving release binary")
    shutil.move(str(binary_src), str(bin_dest))

    # 3. Generate .env
    compose_dir.mkdir(parents=True, exist_ok=True)
    print(f"▶ Writing {env_file}")
    env_file.write_text(
        f"HOST_PATH_BIN={bin_file_name}\n"
        f"API_KEY_CHATGPT={api_key_chatgpt}\n",
        encoding="utf-8",
    )

    # 4. Restart compose stack
    print("▶ Bringing containers up")
    run(
        [
            "docker",
            "compose",
            "up",
            "-d",
            "--build",
            "--force-recreate",
            "--remove-orphans",
        ],
        cwd=compose_dir,
    )
    print("Deploy finished")


if __name__ == "__main__":
    main()
