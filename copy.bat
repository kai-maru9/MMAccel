@echo off

chcp 65001

set MMDDIR="MikuMikuDance_v932x64"

echo %MMDDIR%\MMAccel
if not exist "%MMDDIR%\MMAccel" (
    mkdir "%MMDDIR%\MMAccel"
    echo "%MMDDIR%\MMAccelを作成しました"
)

if exist "%MMDDIR%\MMAccel\key_map.json" (
    del "%MMDDIR%\MMAccel\key_map.json"
)

copy /Y "target\%1\d3d9.dll" "%MMDDIR%"
copy /Y "target\%1\mmaccel.dll" "%MMDDIR%\MMAccel"
copy /Y "target\%1\key_config.exe" "%MMDDIR%\MMAccel"
copy /Y "mmaccel\src\mmd_map.json" "%MMDDIR%\MMAccel"
copy /Y "key_config\src\order.json" "%MMDDIR%\MMAccel"
