#!/bin/python3

# A simple script to trigger Cirrus CI builds and download the release artifacts
# through Cirrus CI's GraphQL interface.
#
# Expects the Cirrus CI API key to be set in the CIRRUS_KEY environment variable.
#
# TODO: Explain this in docs how the heck this works.

import os
import json
import sys
import traceback
from textwrap import dedent
from time import sleep, time
from pathlib import Path
from typing import List, Optional, Tuple

from urllib.request import Request, urlopen, urlretrieve

# Form of each task is (TASK_ALIAS, FILE_NAME).
TASKS: List[Tuple[str, str]] = [
    ("freebsd_13_2_build", "bottom_x86_64-unknown-freebsd-13-2.tar.gz"),
    ("freebsd_14_0_build", "bottom_x86_64-unknown-freebsd-14-0.tar.gz"),
    ("linux_2_17_build", "bottom_x86_64-unknown-linux-gnu-2-17.tar.gz"),
]
URL = "https://api.cirrus-ci.com/graphql"
DL_URL_TEMPLATE = "https://api.cirrus-ci.com/v1/artifact/build/%s/%s/binaries/%s"


def make_query_request(key: str, branch: str, build_type: str):
    print("Creating query request.")
    mutation_id = "Cirrus CI Build {}-{}-{}".format(build_type, branch, int(time()))

    # Dumb but if it works...
    config_override = (
        Path(".cirrus.yml").read_text().replace("# -PLACEHOLDER FOR CI-", 'BTM_BUILD_RELEASE_CALLER: "nightly"')
    )
    query = """
        mutation CreateCirrusCIBuild (
            $repo: ID!,
            $branch: String!,
            $mutation_id: String!,
            $config_override: String,
        ) {
            createBuild(
                input: {
                    repositoryId: $repo,
                    branch: $branch,
                    clientMutationId: $mutation_id,
                    configOverride: $config_override
                }
            ) {
                build {
                    id,
                    status
                }
            }
        }
    """
    params = {
        "repo": "6646638922956800",
        "branch": branch,
        "mutation_id": mutation_id,
        "config_override": dedent(config_override),
    }
    data = {"query": dedent(query), "variables": params}
    data = json.dumps(data).encode()

    request = Request(URL, data=data, method="POST")
    request.add_header("Authorization", "Bearer {}".format(key))

    return request


def check_build_status(key: str, id: str) -> Optional[str]:
    query = """
        query BuildStatus($id: ID!) {
            build(id: $id) {
                status
        }
    }
    """
    params = {
        "id": id,
    }

    data = {"query": dedent(query), "variables": params}
    data = json.dumps(data).encode()

    request = Request(URL, data=data, method="POST")
    request.add_header("Authorization", "Bearer {}".format(key))
    with urlopen(request) as response:
        response = json.load(response)
        if response.get("errors") is not None:
            print("There was an error in the returned response.")
            return None

        try:
            status = response["data"]["build"]["status"]
            return status
        except KeyError:
            print("There was an issue with creating a build job.")
            return None


def try_download(build_id: str, dl_path: Path):
    for task, file in TASKS:
        url = DL_URL_TEMPLATE % (build_id, task, file)
        out = os.path.join(dl_path, file)
        print("Downloading {} to {}".format(file, out))
        urlretrieve(url, out)


def main():
    args = sys.argv
    env = os.environ

    key = env["CIRRUS_KEY"]
    branch = args[1]
    dl_path = args[2] if len(args) >= 3 else ""
    dl_path = Path(dl_path)
    build_type = args[3] if len(args) >= 4 else "build"
    build_id = args[4] if len(args) >= 5 else None

    print(f"Running Cirrus script with branch '{branch}'")

    # Check if this build has already been completed before.
    if build_id is not None:
        print("Previous build ID was provided, checking if complete.")
        status = check_build_status(key, build_id)
        if status.startswith("COMPLETE"):
            print("Starting download of previous build ID")
            try_download(build_id, dl_path)
    else:
        # Try up to three times
        MAX_ATTEMPTS = 5
        success = False

        for i in range(MAX_ATTEMPTS):
            if success:
                break
            print("Attempt {}:".format(i + 1))

            with urlopen(make_query_request(key, branch, build_type)) as response:
                response = json.load(response)

                if response.get("errors") is not None:
                    print("There was an error in the returned response.")
                    continue

                try:
                    build_id = response["data"]["createBuild"]["build"]["id"]
                    print("Created build job {}.".format(build_id))
                except KeyError:
                    print("There was an issue with creating a build job.")
                    continue

                # First, sleep 4 minutes, as it's unlikely it'll finish before then.
                SLEEP_MINUTES = 4
                print("Sleeping for {} minutes.".format(SLEEP_MINUTES))
                sleep(60 * SLEEP_MINUTES)
                print("Mandatory nap over. Starting to check for completion.")

                MINUTES = 10
                SLEEP_SEC = 30
                TRIES = int(MINUTES * (60 / SLEEP_SEC))  # Works out to 20 tries.

                for attempt in range(TRIES):
                    print("Checking...")
                    try:
                        status = check_build_status(key, build_id)
                        if status.startswith("COMPLETE"):
                            print("Build complete. Downloading artifact files.")
                            sleep(5)
                            try_download(build_id, dl_path)
                            success = True
                            break
                        else:
                            print("Build status: {}".format(status or "unknown"))
                            if status == "ABORTED":
                                print("Build aborted, bailing.")
                                break
                            elif status.lower().startswith("fail"):
                                print("Build failed, bailing.")
                                break
                            elif attempt + 1 < TRIES:
                                sleep(SLEEP_SEC)
                    except Exception as ex:
                        print("Unexpected error:")
                        print(ex)
                        print(traceback.format_exc())
                        # Sleep for a minute if something went wrong, just in case.
                        sleep(60)
                else:
                    print("Build failed to complete after {} minutes, bailing.".format(MINUTES))

        if not success:
            exit(2)


if __name__ == "__main__":
    main()
