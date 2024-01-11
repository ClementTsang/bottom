#!/bin/bash

set -e

VENV_PATH="./venv/"
PYTHON_CMD=${1:-python}

if [ ! -d $VENV_PATH ]; then
    echo "venv not found, creating one using the command '${PYTHON_CMD}'...";
    $PYTHON_CMD -m venv venv;
    source ./venv/bin/activate;
    pip install --upgrade pip;
    pip install -r requirements.txt;
    ./venv/bin/mkdocs serve;
else
    echo "venv already found.";
    source ./venv/bin/activate;
    pip install --upgrade pip;
    pip install -r requirements.txt;
    ./venv/bin/mkdocs serve;
fi;

