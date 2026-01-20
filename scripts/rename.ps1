# PowerShell script to rename thefuck to oops in all Rust files

$srcPath = "Q:\Software\oops\src"
$testsPath = "Q:\Software\oops\tests"
$benchPath = "Q:\Software\oops\benches"

function Replace-InFile {
    param (
        [string]$Path,
        [string]$Find,
        [string]$Replace
    )

    $content = Get-Content -Path $Path -Raw
    if ($content -match [regex]::Escape($Find)) {
        $content = $content -replace [regex]::Escape($Find), $Replace
        Set-Content -Path $Path -Value $content -NoNewline
        Write-Host "Updated: $Path"
    }
}

# Get all .rs files
$files = @()
$files += Get-ChildItem -Path $srcPath -Recurse -Filter "*.rs" -File
$files += Get-ChildItem -Path $testsPath -Recurse -Filter "*.rs" -File
$files += Get-ChildItem -Path $benchPath -Recurse -Filter "*.rs" -File

Write-Host "Found $($files.Count) Rust files to process"

foreach ($file in $files) {
    $content = Get-Content -Path $file.FullName -Raw
    $modified = $false

    # Replace use thefuck:: with use oops::
    if ($content -match "use thefuck::") {
        $content = $content -replace "use thefuck::", "use oops::"
        $modified = $true
    }

    # Replace thefuck:: crate references with oops::
    if ($content -match "thefuck::") {
        $content = $content -replace "thefuck::", "oops::"
        $modified = $true
    }

    # Replace cargo_bin("thefuck") with cargo_bin("oops")
    if ($content -match 'cargo_bin\("thefuck"\)') {
        $content = $content -replace 'cargo_bin\("thefuck"\)', 'cargo_bin("oops")'
        $modified = $true
    }

    if ($modified) {
        Set-Content -Path $file.FullName -Value $content -NoNewline
        Write-Host "Updated: $($file.FullName)"
    }
}

Write-Host "Done!"
