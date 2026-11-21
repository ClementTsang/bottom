#!/bin/bash

# Used to serve a versioned version of the docs locally. Note this
# does NOT reflect local changes.

set -e

VENV_PATH="./.venv/"
PYTHON_CMD=${1:-python}

if [ ! -d $VENV_PATH ]; then
    echo "venv not found, creating one using the command '${PYTHON_CMD}'...";
    $PYTHON_CMD -m venv .venv;
    source $VENV_PATH/bin/activate;
    pip install --upgrade pip;
    pip install -r requirements.txt;
    $VENV_PATH/bin/mike serve;
else
    echo "venv already found.";
    source $VENV_PATH/bin/activate;
    pip install --upgrade pip;
    pip install -r requirements.txt;
    $VENV_PATH/bin/mike serve;
fi;

