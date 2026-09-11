{
  description = "A Nix-flake-based Rust development environment for tmux-nav";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forEachSystem = f: nixpkgs.lib.genAttrs supportedSystems (system: f {
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
      });
    in
    {
      packages = forEachSystem ({ pkgs }: {
        default = pkgs.rustPlatform.buildRustPackage {
          pname = "tmux-nav";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

            postInstall = ''
            mkdir -p $out/bin
            cat << 'EOF' > $out/bin/tmux-nav-split
            #!/usr/bin/env bash
            W_ID=$(tmux new-window -P -F "#{window_id}")
            P_SHELL=$(tmux display-message -t "$W_ID.0" -p "#{pane_id}")
            
            # Instantly split and launch the Rust app natively (no typing!)
            P_NAV=$(tmux split-window -t "$W_ID" -h -l 15% -b -P -F "#{pane_id}" "tmux-nav --target-pane '$P_SHELL'")
            
            # Hide the dividing border for this specific window
            tmux set-window-option -t "$W_ID" pane-border-style "fg=#282726,bg=#282726"
            tmux set-window-option -t "$W_ID" pane-active-border-style "fg=#282726,bg=#282726"
            
            tmux select-pane -t "$P_SHELL"
            EOF
            chmod +x $out/bin/tmux-nav-split
          '';
        };
      });

      devShells = forEachSystem ({ pkgs }: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" "rust-analyzer" ];
            })
            cargo
            tmux
            zsh
            eza
          ];

          env = {
            RUST_BACKTRACE = "1";
          };
        };
      });
    };
}
