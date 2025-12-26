import os

import mkdocs.plugins


@mkdocs.plugins.event_priority(-100)
def on_config(config):
    version = os.environ.get("MIKE_DOCS_VERSION")

    if version == "nightly:"
        extra = config.get("extra", {})
        extra["nightly"] = True

