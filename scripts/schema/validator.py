#!/bin/python3

# A simple script to validate that a schema is valid for a file.

import argparse
import toml
import jsonschema_rs


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
        "--should_fail",
        required=False,
        action="store_true",
        help="Whether the checked file should fail.",
    )
    args = parser.parse_args()

    file = args.file
    schema = args.schema
    should_fail = args.should_fail

    with open(file) as f, open(schema) as s:
        try:
            validator = jsonschema_rs.JSONSchema.from_str(s.read())
        except:
            print("Coudln't create validator.")
            exit()

        is_valid = validator.is_valid(toml.load(f))
        if is_valid:
            if should_fail:
                print("Fail!")
                exit(1)
            else:
                print("All good!")
        else:
            if should_fail:
                print("Caught error, good!")
            else:
                print("Fail!")
                exit(1)


if __name__ == "__main__":
    main()
