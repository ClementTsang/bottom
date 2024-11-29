#!/bin/python3

# A simple script to validate that a schema is valid for a file.

import argparse
import tomllib
import jsonschema_rs
import re
import traceback


def main():
    parser = argparse.ArgumentParser(
        description="Validates a file against a JSON schema"
    )
    parser.add_argument(
        "-f", "--file", type=str, required=True, help="The file to check."
    )
    parser.add_argument(
        "-s", "--schema", type=str, required=True, help="The schema to use."
    )
    parser.add_argument(
        "--uncomment",
        required=False,
        action="store_true",
        help="Uncomment the settings inside the file.",
    )
    parser.add_argument(
        "--should_fail",
        required=False,
        action="store_true",
        help="Whether the checked file should fail.",
    )
    args = parser.parse_args()

    file = args.file
    schema = args.schema
    should_fail = args.should_fail
    uncomment = args.uncomment

    with open(file, "rb") as f, open(schema) as s:
        try:
            validator = jsonschema_rs.validator_for(s.read())
        except:
            print("Couldn't create validator.")
            exit()

        if uncomment:
            read_file = f.read().decode("utf-8")
            read_file = re.sub(r"^#([a-zA-Z\[])", r"\1", read_file, flags=re.MULTILINE)
            read_file = re.sub(
                r"^#(\s\s+)([a-zA-Z\[])", r"\2", read_file, flags=re.MULTILINE
            )
            print(f"uncommented file: \n{read_file}\n=====\n")

            toml_str = tomllib.loads(read_file)
        else:
            toml_str = tomllib.load(f)

        try:
            validator.validate(toml_str)
            if should_fail:
                print("Fail! Should have errored.")
                exit(1)
            else:
                print("All good!")
        except jsonschema_rs.ValidationError as err:
            print(f"Caught error: `{err}`")
            print(traceback.format_exc())

            if should_fail:
                print("Caught error, good!")
            else:
                print("Fail!")
                exit(1)


if __name__ == "__main__":
    main()
