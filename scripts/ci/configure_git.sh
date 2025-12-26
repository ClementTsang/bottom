#!/bin/bash

git config --global user.name ${GIT_USER}
git config --global user.email ${GIT_EMAIL}
echo Name: $(git config --get user.name)
echo Email: $(git config --get user.email)
