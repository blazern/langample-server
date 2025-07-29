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
        description="Fresh-clone repo, drop fat JAR in place, write .env, restart compose.",
    )
    parser.add_argument(
        "--jar-path",
        required=True,
        type=Path,
        help="Absolute path to the fat-JAR on the server",
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

    jar_src: Path = args.jar_path.resolve()
    api_key_chatgpt: str = args.api_key_chatgpt
    deploy_dir: Path = args.deploy_dir
    repo_url: str = args.repo_url

    compose_dir = deploy_dir / "docker"
    jar_dest = deploy_dir / "backend" / "backend.jar"
    env_file = compose_dir / ".env"

    # 1. Fresh repo
    print(f"▶ Cleaning {deploy_dir}")
    shutil.rmtree(deploy_dir, ignore_errors=True)

    print("▶ Cloning repo")
    run(["git", "clone", "--depth", "1", repo_url, str(deploy_dir)])

    # 2. Move fat-JAR
    jar_dest.parent.mkdir(parents=True, exist_ok=True)
    print("▶ Moving fat JAR")
    shutil.move(str(jar_src), str(jar_dest))

    # 3. Generate .env
    compose_dir.mkdir(parents=True, exist_ok=True)
    print(f"▶ Writing {env_file}")
    env_file.write_text(
        f"HOST_PATH_JAR={os.path.relpath(jar_dest, compose_dir)}\n"
        f"api_key_chatgpt={api_key_chatgpt}\n",
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
