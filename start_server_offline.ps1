$env:SQLX_OFFLINE = "true"
cd d:\studying\Codecargo\CarpTMS\My_server
cargo run --bin carptms_server 2>&1 | Tee-Object -FilePath "d:\studying\Codecargo\CarpTMS\server_output.txt"
