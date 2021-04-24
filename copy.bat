@echo off

set MMDDIR="MikuMikuDance_v932x64"

if not exist "%MMDDIR%\MMAccelEx" (
    mkdir "%MMDDIR%\MMAccelEx"
)

copy /Y /B "target\%1\d3d9.dll" "%MMDDIR%"
copy /Y /B "target\%1\mmaccel_ex.dll" "%MMDDIR%\MMAccelEx"