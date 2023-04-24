#Requires -Version 7

<#
.SYNOPSIS
Builds the `fallible_vec` crate, runs tests, checks formatting, runs clippy.

.PARAMETER BuildLocked
Adds `--locked` to the build commands to prevent the `Cargo.lock` file from being updated. This is
useful for CI builds.

.NOTES
See README.md for details on the environment that this script expects.
#>
param (
    [Parameter(Mandatory = $false)]
    [switch]
    $BuildLocked
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$lockedArg = $BuildLocked ? '--locked' : $null

function Invoke-CheckExitCode([string] $Description, [scriptblock]$ScriptBlock) {
    Write-Host "==== $Description ===="
    & $ScriptBlock
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}

function Invoke-WithEnvironment([System.Collections.IDictionary] $Environment, [scriptblock]$ScriptBlock) {
    try {
        # Set the environment.
        foreach ($item in $Environment.GetEnumerator()) {
            Set-Item -Path $item.Key -Value $item.Value
        }
        & $ScriptBlock
    } finally {
        # Cleanup the environment afterwards.
        foreach ($item in $Environment.Keys) {
            if (Test-Path -Path $item) {
                Remove-Item $item
            }
        }
    }
}

# Verify that all sources files have the copyright header.
[string[]] $copyrightHeader = @("// Copyright (c) Microsoft Corporation.", "// Licensed under the MIT license.")
[bool] $hadMissingCopyright = $false
foreach ($file in (Get-ChildItem -Path (Join-Path $PSScriptRoot 'src') -Filter '*.rs' -Recurse)) {
    $contents = Get-Content -Path $file -TotalCount $copyrightHeader.Length
    if ($null -ne (Compare-Object -ReferenceObject $copyrightHeader -DifferenceObject $contents)) {
        $hadMissingCopyright = $true
        $fileName = $file.FullName
        Write-Error "'$fileName' is missing the copyright header." -ErrorAction Continue
    }
}
if ($hadMissingCopyright) {
    $mergedCopyrightHeader = $copyrightHeader | Join-String -Separator "`n"
    Write-Error "One or more files was missing the copyright header. To fix this, add the copyright header to any non-compliant files:`n$mergedCopyrightHeader"
    exit 1
}

Invoke-WithEnvironment `
    -Environment @{
        # Enable unstable features on stable toolchain.
        'env:RUSTC_BOOTSTRAP' = '1';
        # Fail 'cargo doc' on warnings.
        'env:RUSTDOCFLAGS' = '-D warnings';
        # Fail 'cargo build' on warnings.
        'env:RUSTFLAGS' = '-D warnings';
    } `
    -ScriptBlock {
        #
        # Check that enabling various feature combinations works.
        #
        Invoke-CheckExitCode 'Build default' { cargo build $lockedArg }
        Invoke-CheckExitCode 'Build allocator_api only' { cargo build $lockedArg --no-default-features --features allocator_api }
        Invoke-CheckExitCode 'Build use_unstable_apis only' { cargo build $lockedArg --no-default-features --features use_unstable_apis }

        #
        # Run tests
        #
        Invoke-CheckExitCode 'Test' { cargo test --locked }

        #
        # Lint and check formatting.
        #
        Invoke-CheckExitCode 'Clippy' { cargo clippy --locked -- -D warnings }
        Invoke-CheckExitCode 'Check format' { cargo fmt --check }

        #
        # Check docs
        #
        Invoke-CheckExitCode 'Check docs' { cargo doc --locked }

        #
        # Verify that we can build with #[cfg(no_global_oom_handling)] enabled.
        #

        # Find target (required for `build-std`).
        [string] $target = ''
        if ($Global:IsWindows) {
            $target = 'x86_64-pc-windows-msvc'
        } elseif ($Global:IsLinux) {
            $target = 'x86_64-unknown-linux-gnu'
        } elseif ($Global:IsMacOS) {
            $target = 'x86_64-apple-darwin'
        } else {
            throw 'Unknown OS - Only Windows, Linux and MacOS are supported'
        }
        Invoke-WithEnvironment `
            -Environment @{
                # `no_global_oom_handling` disable all infallible allocation functions
                # in the standard library.
                'env:RUSTFLAGS' = '--cfg no_global_oom_handling';
            } `
            -ScriptBlock {
                Invoke-CheckExitCode 'Build no_global_oom_handling' { cargo build $lockedArg -Z build-std=core,alloc --target $target }
            }
}

# Build with no features enabled (should work on the non-nightly compiler).
Invoke-CheckExitCode 'Build no features' { cargo build $lockedArg --no-default-features }

# Run tests under miri
Invoke-CheckExitCode 'Install miri' { rustup toolchain install nightly --component miri }
Invoke-CheckExitCode 'Setup miti' { cargo +nightly miri setup }
Invoke-CheckExitCode 'Miri test' { cargo +nightly miri test }
