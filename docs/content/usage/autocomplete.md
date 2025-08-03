# Auto-Complete

The release binaries in [the releases page](https://github.com/ClementTsang/bottom/releases) are packaged with
shell auto-completion files for Bash, Zsh, fish, Powershell, Elvish, Fig, and Nushell. To install them:

- For Bash, move `btm.bash` to `$XDG_CONFIG_HOME/bash_completion or /etc/bash_completion.d/`.
- For Zsh, move `_btm` to one of your `$fpath` directories.
- For fish, move `btm.fish` to `$HOME/.config/fish/completions/`.
- For PowerShell, add `_btm.ps1` to your PowerShell [profile](<https://docs.microsoft.com/en-us/previous-versions//bb613488(v=vs.85)>).
- For Elvish, the completion file is `btm.elv`.
- For Fig, the completion file is `btm.ts`.
- For Nushell, source `btm.nu`.

The individual auto-completion files are also included in the stable/nightly releases as `completion.tar.gz` if needed.
