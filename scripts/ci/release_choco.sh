#!/bin/bash

curl -X POST https://api.github.com/repos/ClementTsang/choco-bottom/dispatches \
    -H 'Accept: application/vnd.github.everest-preview+json' \
    -u ${{ secrets.BOTTOM_PACKAGE_DEPLOYMENT }} \
    --data '{ "event_type": "update", "client_payload": { "version": "'"$RELEASE_VERSION"'" } }'
