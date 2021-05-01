$dir = "MikuMikuDance_v932x64"
$target = Join-Path "target" $args[0]
$mmaccel_dir = Join-Path $dir "MMAccel/"

if (!(Test-Path -Path $mmaccel_dir)) {
    New-Item $mmaccel_dir -ItemType Directory
}

Copy-Item (Join-Path $target "d3d9.dll") $dir
Copy-Item (Join-Path $target "mmaccel.dll") $mmaccel_dir
Copy-Item (Join-Path $target "key_config.exe") $mmaccel_dir
Copy-Item "mmaccel/src/mmd_map.json" $mmaccel_dir
Copy-Item "key_config/src/order.json" $mmaccel_dir
