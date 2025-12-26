import os
import sys

import mkdocs.plugins


@mkdocs.plugins.event_priority(-100)
def on_config(config):
    print("Running nightly banner hook...", file=sys.stderr)

    # From https://github.com/jimporter/mike/blob/3351d5feabff8ee107f4ad6d1f86055843c7dbf1/mike/mkdocs_utils.py#L13
    version = os.environ.get("MIKE_DOCS_VERSION")
    print(f"Version: {version}", file=sys.stderr)

    if version == "nightly":
        extra = config.get("extra", {})
        extra["nightly"] = True

