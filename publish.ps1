if (!(Test-Path "package")) {
    Write-Host "packageフォルダがありません"
    exit
}

$version = Get-Content "version"

git tag -a $version -m "MMAccel $version"
git push origin
git push origin $version
