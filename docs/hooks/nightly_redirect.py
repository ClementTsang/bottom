import mkdocs.plugins
import urllib.request
import json


# Based on https://github.com/squidfunk/mkdocs-material/discussions/3758#discussioncomment-4397373

@mkdocs.plugins.event_priority(-50)
def on_config(config):
    print("Running nightly release redirect hook...")
    try:
        with urllib.request.urlopen("https://api.github.com/repos/ClementTsang/bottom/releases") as response:
            raw_data = response.read()
            data = json.loads(raw_data.decode('utf-8'))

            first_nightly = next(release for release in data if "nightly-" in release["tag_name"])
            nightly_tag_name = first_nightly["tag_name"]

            redirect_plugin = config.get('plugins', {}).get('redirects')
            redirects = redirect_plugin.config.get('redirect_maps', {})

            nightly_release_url = f"https://github.com/ClementTsang/bottom/releases/tag/{nightly_tag_name}"

            print(f"Updated nightly release redirect to point to {nightly_release_url}")
            redirects["nightly-release.md"] = nightly_release_url
    except Exception as e:
        print(f"error adjusting redirect, falling back to general releases page: {e}")
