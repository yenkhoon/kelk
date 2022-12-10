#! /usr/bin/env python3

"""This is a script for publishing the Kelk crates to crates.io.
Crates in workspace shouuld be listed in order of publish.
It should be run in the root of the project like:
```
python3 .github/workflows/publish.py --help
```
Please lint with pylint and format with black.
```
pylint .github/workflows/publish.py
black .github/workflows/publish.py
```
"""

import argparse
import re
import subprocess
import time
import sys
from pprint import pprint

try:
    # try import tomli
    import tomli
except ModuleNotFoundError:
    print("Please install tomli, `pip3 install tomli`")
    sys.exit(1)


def get_latest_release_version():
    """Get the lastest release version.
    The version extracts from the most recent git tag.
    Git tag should start with `v` following the semver."""
    try:
        git_tag = str(
            subprocess.check_output(
                ["git", "describe", "--abbrev=0"], stderr=subprocess.STDOUT
            )
        ).strip("'b\\nv")
    except subprocess.CalledProcessError as exc_info:
        raise Exception(str(exc_info.output)) from exc_info
    return git_tag


def get_latest_version_for_crate(crate_name: str) -> str:
    """Fetches the latest published version of a given crate name."""
    output = subprocess.run(
        ["cargo", "search", crate_name], capture_output=True, check=True
    )
    rexp_src = f'^{crate_name} = "([^"]+)"'
    prog = re.compile(rexp_src)
    haystack = output.stdout.decode("utf-8")
    for line in haystack.splitlines():
        result = prog.match(line)
        if result:
            return result.group(1)
    return ""


class Crate:
    """Represents a crate that is to be published to crates.io."""

    def __init__(self, toml: str, verbose: bool = False):
        with open(toml, "rb") as file:
            data = tomli.load(file)

        self.toml = data
        self.verbose = verbose

    def __str__(self):
        return f"{self.name}: {self.version}"

    @property
    def name(self) -> str:
        """Return the crate's name according to its manifest."""
        return self.toml["package"]["name"]

    @property
    def version(self) -> str:
        """Return the crate's version according to its manifest."""
        return self.toml["package"]["version"]

    def publish(self, dry_run: bool):
        """Publish this crate to crates.io."""
        command = ["cargo", "publish", "-p", self.name]
        if dry_run:
            print(f"In dry-run: not publishing crate `{self.name}`")
            command.append("--dry-run")

        command.append("--dry-run")
        if self.verbose:
            print(*command)
        output = subprocess.run(command, check=True)
        if self.verbose:
            print(output)
            print("{self.name} sucessfully published in crates.io")

    def is_already_published(self) -> bool:
        """Checks if the crate name is already published
        with the same version string."""
        found_string: str = get_latest_version_for_crate(self.name)
        if found_string == "":
            return False

        return self.version == found_string


class Workspace:
    """A class representing workspace.
    The crates in the workspace should be listed in order of publish.
    """

    def __init__(self, verbose=False):
        # open workspace Cargo.toml
        with open("Cargo.toml", "rb") as file:
            data = tomli.load(file)

        crates = []
        for member in data["workspace"]["members"]:
            toml = member + "/Cargo.toml"
            if verbose:
                print(f"Parsing {toml}")
            crates.append(Crate(toml, verbose))

        self.verbose: bool = verbose
        self.crates = crates

    def check(self):
        """Ensures that:
        1- All crates have the same version as the lates release version
        2- No crate with the same version has published yet
        """
        ver = get_latest_release_version()
        if self.verbose:
            print(f"Latest release version is {ver}")

        for crate in self.crates:
            if ver != crate.version:
                print(f"{crate.name} version should be {ver}")
                return 1

            if crate.is_already_published():
                print(f"{crate.name} is already published with version {ver}")
                return 2

        return 0

    def publish(self, dry_run: bool):
        """Publish all crates in workspace."""

        if self.verbose and dry_run:
            print("(Dry run, not actually publishing anything)")

        if self.verbose:
            print("Publishing order:")
            for crate in self.crates:
                pprint(crate.name)

        for crate in self.crates:
            print(f"Publishing `{crate.name}`...")
            crate.publish(dry_run)

            # sleep for 16 seconds between crates to ensure the crates.io
            # index has time to update
            if not dry_run:
                print(
                    "Sleeping for 16 seconds to allow the `crates.io` index to update..."
                )
                time.sleep(16)
            else:
                print("In dry-run: not sleeping for crates.io to update.")


def main():
    """Main executable function."""
    parser = argparse.ArgumentParser(description="Publish the Kelk crates to crates.io")

    subparsers = parser.add_subparsers(dest="subcommand")
    check_cmd = subparsers.add_parser(
        "check",
        help="""Check if the crates versions are same as release version.""",
    )
    check_cmd.add_argument(
        "-v",
        "--verbose",
        action="store_true",
        default=False,
        help="Be verbose.",
    )

    publish_cmd = subparsers.add_parser(
        "publish", help="Publish Wasmer crates to crates.io."
    )
    publish_cmd.add_argument(
        "-v",
        "--verbose",
        action="store_true",
        default=False,
        help="Be verbose.",
    )
    publish_cmd.add_argument(
        "--dry-run",
        default=True,
        action="store_true",
        help="""Run the script without actually publishing anything to
        crates.io""",
    )

    args = parser.parse_args()
    verbose = args.verbose
    if args.subcommand == "check":
        publisher = Workspace(verbose=verbose)
        return publisher.check()

    if args.subcommand == "publish":
        verbose = args.verbose
        publisher = Workspace(verbose=verbose)
        return publisher.publish(args.dry_run)

    return 0


if __name__ == "__main__":
    main()
