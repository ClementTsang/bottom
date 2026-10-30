$ErrorActionPreference = 'Stop'

$msys2 = 'C:\msys64'
$package = 'mingw-w64-clang-aarch64-clang=23.1.4-1'
& "$msys2\usr\bin\bash.exe" -lc 'pacman --noconfirm -Sy'
if ($LASTEXITCODE -ne 0) {
    throw 'Failed to synchronize the MSYS2 package databases'
}
& "$msys2\usr\bin\bash.exe" -lc "pacman --noconfirm -S --needed '$package'"
if ($LASTEXITCODE -ne 0) {
    throw 'Failed to install the CLANGARM64 toolchain'
}

$bin = "$msys2\clangarm64\bin"
$compiler = "$bin\aarch64-w64-mingw32-clang.exe"
if (-not (Test-Path $compiler)) {
    throw "The CLANGARM64 compiler was not found at $compiler"
}

$bin >> $env:GITHUB_PATH
& $compiler --version
