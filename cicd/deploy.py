#!/usr/bin/env python3
from __future__ import annotations
import argparse
import os
import shutil
import subprocess
import json
import time
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
        "--graphql-parent-path",
        required=True,
        help="Parent path prefix for GraphQL endpoints (e.g. /langample/)",
    )
    parser.add_argument(
        "--panlex-sqlite-db-path",
        required=True,
        help="Absolute path to panlex.sqlite on the server (e.g. ~/panlex.sqlite)",
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
    graphql_parent_path: str = args.graphql_parent_path
    panlex_sqlite_db_path: Path = Path(args.panlex_sqlite_db_path).expanduser().resolve()
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
        f"API_KEY_CHATGPT={api_key_chatgpt}\n"
        f"GRAPHQL_PARENT_PATH={graphql_parent_path}\n"
        f"PANLEX_SQLITE_DB_PATH={panlex_sqlite_db_path}\n",
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

    print("▶ Checking containers' health")
    assert_containers_healthy(compose_dir)

    print("Deploy finished")


def assert_containers_healthy(
    compose_dir: Path,
    timeout: int = 60,
    stable_seconds: int = 15,
) -> None:
    """
    Waits until every container is continuously healthy.
    """
    deadline = time.time() + timeout
    stable_since: float | None = None

    while time.time() < deadline:
        ps = subprocess.run(
            ["docker", "compose", "ps", "--format", "json"],
            cwd=compose_dir,
            capture_output=True,
            text=True,
            check=True,
        )

        services_json = [
            json.loads(line) for line in ps.stdout.splitlines() if line.strip()
        ]

        states = {
            s["Name"]: (s["State"], s.get("Health", ""))
            for s in services_json
        }

        exited_or_restart = [
            n for n, (st, _) in states.items() if st in ("exited", "restarting")
        ]
        unhealthy = [
            n for n, (_, health) in states.items() if health == "unhealthy"
        ]

        if exited_or_restart or unhealthy:
            raise RuntimeError(
                f"Faulty containers – exited/restarting: {exited_or_restart}, "
                f"unhealthy: {unhealthy}"
            )

        all_ok = all(
            st == "running" and (health in ("", "healthy"))
            for st, health in states.values()
        )

        if all_ok:
            if stable_since is None:
                stable_since = time.time()
            elif time.time() - stable_since >= stable_seconds:
                print(
                    f"✔ All containers running and healthy "
                    f"(stable ≥ {stable_seconds}s)"
                )
                return
        else:
            stable_since = None

        time.sleep(2)

    raise TimeoutError(
        f"Containers not healthy after {timeout}s: {states}"
    )


if __name__ == "__main__":
    main()
