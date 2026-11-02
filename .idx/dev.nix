{ pkgs, ... }: {
  channel = "stable-24.05";
  packages = [
    pkgs.rustup
    pkgs.gcc
    pkgs.gnumake
    pkgs.pkg-config
    pkgs.curl
    pkgs.git
    pkgs.tmux
    pkgs.python311
  ];
  env = {
    # 可於 Firebase Studio 終端 export GEMINI_API_KEYS="key1,key2" 或 GROQ_API_KEY
    RUST_BACKTRACE = "1";
  };
  idx = {
    extensions = [
      "rust-lang.rust-analyzer"
      "tamasfe.even-better-toml"
    ];
    workspace = {
      onCreate = {
        rust-init = "rustup default stable && cargo test --no-run";
      };
      onStart = {
        guard-ready = "echo '🌾 rust-bricks 雲端同步快農環境已就緒！輸入 cargo run -- farm <spec> <module> [--push] 即可開農'";
      };
    };
  };
}
