cargo build --all --features hot_reload_libs \
  && RUST_BACKTRACE=1 cargo watch -i "*/.cache/**" -i "*/ec3api/**" -i "*/view/**" -x "run --features hot_reload_libs"
