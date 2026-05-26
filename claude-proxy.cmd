@echo off
setlocal

:: Set Claude specific proxy variables
set HTTP_PROXY=http://127.0.0.1:10808
set HTTPS_PROXY=http://127.0.0.1:10808
set NODE_TLS_REJECT_UNAUTHORIZED=0

echo [Cascade Engine] Proxy environment set for Claude.
claude %*

endlocal
