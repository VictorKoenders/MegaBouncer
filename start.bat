start "Server" /D server cargo run
start "TCP Connector" /D tcp_connector cargo run
start "IRC Connector" /D irc_connector cargo run
start "Logger" /D test_client cargo run