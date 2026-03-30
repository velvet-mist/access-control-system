# Keyence Command Logging TODO

## Steps:
- [x] 1. Update Cargo.toml: Add `serde_json = "1.0"`, `chrono = { version = "0.4", features = ["serde"] }`
- [x] 2. Update src/config.rs: Add `keyence_command_log: String,` loaded from env `KEYENCE_COMMAND_LOG` default "keyence_commands.log"
- [x] 3. Update src/tcp_handler/tcp.rs: Add logger init (Arc<Mutex<tokio::fs::File>>), log JSON {timestamp, peer, command, is_protected, access_decision} on recv, {response} on reply.
- [ ] 4. Update src/main.rs: Print `println!("Keyence command log: {}", cfg.keyence_command_log);`
- [ ] 5. Suggest .env: `KEYENCE_COMMAND_LOG=keyence_commands.log`
- [x] 6. cargo build &amp;&amp; test (telnet to proxy port, check log)

Track progress by editing this file.

