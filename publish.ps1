if (!(Test-Path "package")) {
    Write-Host "packageフォルダがありません"
    exit
}

$version = Get-Content "version"

git commit --allow-empty -am "MMAccel $version"
git tag $version
git push origin
git push origin $version
