# Fix shell aliases to call 'oops' instead of 'thefuck'

$shellFiles = @(
    "Q:\Software\oops\src\shells\bash.rs",
    "Q:\Software\oops\src\shells\zsh.rs",
    "Q:\Software\oops\src\shells\fish.rs",
    "Q:\Software\oops\src\shells\powershell.rs",
    "Q:\Software\oops\src\shells\tcsh.rs"
)

foreach ($file in $shellFiles) {
    if (Test-Path $file) {
        $content = Get-Content -Path $file -Raw

        # Replace thefuck binary call in shell aliases (but not comments/docs)
        # Pattern: 'thefuck ' followed by placeholder or args
        $content = $content -replace '(\s+)thefuck (\{placeholder\}|\$)', '$1oops $2'
        $content = $content -replace '`thefuck ', '`oops '
        $content = $content -replace '\$\(thefuck', '$(oops'

        Set-Content -Path $file -Value $content -NoNewline
        Write-Host "Updated: $file"
    }
}

Write-Host "Done!"
