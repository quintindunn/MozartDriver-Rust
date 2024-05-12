@echo off
set exe_path="./mozart_driver.exe"
echo "Execution started"
set start_time=%time%

REM Run the executable
%exe_path%

set end_time=%time%
echo "Execution finished"

REM Calculate the execution time in milliseconds
for /f "tokens=1-4 delims=:.," %%a in ("%start_time%") do (
    set /a start_ms=%%a*3600000 + %%b*60000 + %%c*1000 + %%d
)
for /f "tokens=1-4 delims=:.," %%a in ("%end_time%") do (
    set /a end_ms=%%a*3600000 + %%b*60000 + %%c*1000 + %%d
)

set /a elapsed_ms=end_ms - start_ms
echo "Execution time: %elapsed_ms% milliseconds"
pause