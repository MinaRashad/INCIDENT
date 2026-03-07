@echo off
del /f /q "*.db*"
echo Save file deleted
pause

target\debug\INCIDENT.exe --skip