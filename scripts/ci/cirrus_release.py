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
    ("linux_2_17_build", "bottom_x86_64-unknown-linux-gnu-2-17.tar.gz"),
]
URL = "https://api.cirrus-ci.com/graphql"
DL_URL_TEMPLATE = "https://api.cirrus-ci.com/v1/artifact/build/%s/%s/binaries/%s"


def make_query_request(key: str, branch: str, mutation_id: str):
    print("Creating query request.")

    # Dumb but if it works...
    config_override = (
        Path(".cirrus.yml")
        .read_text()
        .replace("# -PLACEHOLDER FOR CI-", 'BTM_BUILD_RELEASE_CALLER: "ci"')
    )

    query = """
        mutation CreateCirrusCIBuild (
            $repo: ID!,
            $branch: String!,
            $mutation_id: String!,
            $config_override: String,
        ) {
            createBuild (
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


def check_build_status(key: str, build_id: str) -> Optional[str]:
    query = """
        query BuildStatus($id: ID!) {
            build(id: $id) {
                status
            }
        }
    """

    params = {
        "id": build_id,
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
            print("There was an issue with checking the build status.")
            return None


def check_build_tasks(key: str, build_id: str) -> Optional[List[str]]:
    query = """
        query Build($id:ID!) {
            build(id:$id){
                tasks {
                    id
                }
            }
        }
    """

    params = {
        "id": build_id,
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
            tasks = [task["id"] for task in response["data"]["build"]["tasks"]]
            return tasks
        except KeyError:
            print("There was an issue with getting the list of task ids.")
            return None


def stop_build_tasks(key: str, task_ids: List[str], mutation_id: str) -> bool:
    query = """
        mutation StopCirrusCiTasks (
            $task_ids: [ID!]!,
            $mutation_id: String!,
        ) {
                batchAbort (
                input: {
                        taskIds: $task_ids,
                        clientMutationId: $mutation_id
                }
            ) {
                tasks {
                    id
                }
            }
        }
    """

    params = {
        "task_ids": task_ids,
        "mutation_id": mutation_id,
    }

    data = {"query": dedent(query), "variables": params}
    data = json.dumps(data).encode()

    request = Request(URL, data=data, method="POST")
    request.add_header("Authorization", "Bearer {}".format(key))

    with urlopen(request) as response:
        response = json.load(response)
        return len(response["data"]["batchAbort"]["tasks"]) == len(task_ids)


def try_download(build_id: str, dl_path: Path):
    for task, file in TASKS:
        url = DL_URL_TEMPLATE % (build_id, task, file)
        out = os.path.join(dl_path, file)
        print(f"Downloading {file} to {out}")
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
        tasks = []
        mutation_id = None

        for i in range(MAX_ATTEMPTS):
            if success:
                break

            print(f"Attempt {i + 1}:")

            if tasks and mutation_id:
                print("Killing previous tasks first...")

                if stop_build_tasks(key, tasks, mutation_id):
                    print("All previous tasks successfully stopped.")
                else:
                    print(
                        "Not all previous tasks stopped. This isn't a problem but it is a waste."
                    )

            tasks = []
            mutation_id = "Cirrus CI Build {}-{}-{}".format(
                build_type, branch, int(time())
            )

            with urlopen(make_query_request(key, branch, mutation_id)) as response:
                response = json.load(response)
                errors = response.get("errors")

                if errors is not None:
                    print(f"There was an error in the returned response: {str(errors)}")
                    continue

                try:
                    build_id = response["data"]["createBuild"]["build"]["id"]
                    print(f"Created build job {build_id}.")
                except KeyError:
                    print("There was an issue with creating a build job.")
                    continue

                # First, sleep X minutes total, as it's unlikely it'll finish before then.
                SLEEP_MINUTES = 4
                print(f"Sleeping for {SLEEP_MINUTES} minutes.")

                # Sleep and check for tasks out every 10 seconds
                for _ in range(SLEEP_MINUTES * 6):
                    sleep(10)
                    if not tasks:
                        tasks = check_build_tasks(key, build_id)

                MINUTES = 10
                SLEEP_SEC = 30
                TRIES = int(MINUTES * (60 / SLEEP_SEC))  # Works out to 20 tries.

                print(f"Mandatory nap over. Checking for completion for {MINUTES} min.")

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
                            print(f"Build status: {(status or 'unknown')}")

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
                    print(f"Build failed to complete after {MINUTES} minutes, bailing.")

        if not success:
            exit(2)


if __name__ == "__main__":
    main()
