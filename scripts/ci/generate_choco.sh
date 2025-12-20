#!/bin/bash

python "./scripts/windows/choco/choco_packager.py" \
    "./release/bottom_x86_64-pc-windows-msvc.zip" \
    ${{ env.RELEASE_VERSION }} \
    "./scripts/windows/choco/bottom.nuspec.template" \
    "./scripts/windows/choco/chocolateyinstall.ps1.template" \
    "bottom.nuspec" \
    "tools/chocolateyinstall.ps1" \
    "tools/"
zip -r choco.zip "bottom.nuspec" "tools"