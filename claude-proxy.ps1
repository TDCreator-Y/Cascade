# Set Claude specific proxy variables
$env:HTTP_PROXY="http://127.0.0.1:10808"
$env:HTTPS_PROXY="http://127.0.0.1:10808"
$env:NODE_TLS_REJECT_UNAUTHORIZED="0"

Write-Host "[Cascade Engine] Proxy environment set for Claude." -ForegroundColor Cyan
claude $args
