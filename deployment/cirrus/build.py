#!/bin/python3

import json
import sys
from textwrap import dedent
from time import sleep, time
from pathlib import Path

from urllib.request import Request, urlopen, urlretrieve

URL = "https://api.cirrus-ci.com/graphql"
TASKS = [
    ("freebsd_build", "bottom_x86_64-unknown-freebsd.tar.gz"),
    ("macos_build", "bottom_aarch64-apple-darwin.tar.gz"),
]
DL_URL_TEMPLATE = "https://api.cirrus-ci.com/v1/artifact/build/%s/%s/binaries/%s"


def make_query_request(key: str, branch: str, build_type: str):
    print("Creating query request.")
    mutation_id = "Cirrus CI Build {}-{}-{}".format(build_type, branch, int(time()))
    query = """
        mutation CreateCirrusCIBuild (
            $repo: ID!,
            $branch: String!,
            $mutation_id: String!
        ) {
            createBuild(
                input: {
                    repositoryId: $repo,
                    branch: $branch,
                    clientMutationId: $mutation_id,
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
    }
    data = {"query": dedent(query), "variables": params}
    data = json.dumps(data).encode()

    request = Request(URL, data=data, method="POST")
    request.add_header("Authorization", "Bearer {}".format(key))

    return request


def check_build_status(key: str, id: str) -> bool:
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
            return False

        try:
            status = response["data"]["build"]["status"]
            return status == "COMPLETED"
        except KeyError:
            print("There was an issue with creating a build job.")
            return False

    return False


def main():
    SLEEP_MINUTES = 4
    args = sys.argv

    if len(args) < 2:
        print("cirrus script requires an argument for the API key")
        exit(1)
    elif len(args) < 3:
        print("cirrus script requires an argument for which branch to build")
        exit(1)

    key = args[1]
    branch = args[2]
    dl_path = args[3] if len(args) >= 4 else ""
    dl_path = Path(dl_path)
    build_type = args[4] if len(args) >= 5 else "build"

    with urlopen(make_query_request(key, branch, build_type)) as response:
        response = json.load(response)

        if response.get("errors") is not None:
            print("There was an error in the returned response.")
            exit(2)

        try:
            build_id = response["data"]["createBuild"]["build"]["id"]
            print("Created build job {}. Sleeping for {} minutes.".format(build_id, SLEEP_MINUTES))
        except KeyError:
            print("There was an issue with creating a build job.")
            exit(3)

        # First, sleep 4 minutes, it's unlikely it'll finish quickly enough.
        sleep(60 * SLEEP_MINUTES)

        # Try for up to 10 minutes, waiting 30 seconds each time.
        # In other words, try 20 times with 30s sleep for completion.

        TRIES = 20
        SLEEP_SEC = 30

        for attempt in range(TRIES):
            if check_build_status(key, build_id):
                print("Downloading artifact files.")
                for (task, file) in TASKS:
                    url = DL_URL_TEMPLATE % (build_id, task, file)
                    out = dl_path / file
                    print("Downloading {} to {}".format(file, out))
                    urlretrieve(url, out)
                exit(0)

            if attempt + 1 < TRIES:
                sleep(SLEEP_SEC)
        else:
            print("Build failed to complete after 10 minutes, bailing.")
            exit(4)


if __name__ == "__main__":
    main()
