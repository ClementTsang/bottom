#!/bin/bash

cd docs
OLD_STABLE_VERSION=$(mike list stable | grep -Po '([0-9]+.[0-9]+.[0-9]+)' | head -n1)
echo ${OLD_STABLE_VERSION}
mike retitle --push stable ${OLD_STABLE_VERSION}
mike deploy --push --update-aliases ${RELEASE_VERSION} stable
mike retitle --push ${RELEASE_VERSION} "${RELEASE_VERSION} (stable)"
