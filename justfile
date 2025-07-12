default:
  @just --list

check:
  just fmt --check
  just lint
  just test

fmt check="":
  if [[ "{{check}}" == "--check" ]]; then \
    just fmt-rust --check; \
    just fmt-nix --check; \
    just fmt-dprint --diff; \
  else \
    just fmt-rust; \
    just fmt-nix; \
    just fmt-dprint; \
  fi

fmt-rust *flags:
  cargo fmt {{flags}}

fmt-nix *flags:
  find . -type f -iname '*.nix' | xargs nixpkgs-fmt {{flags}}

fmt-dprint *flags:
  dprint fmt {{flags}}

lint watch="":
  if [[ "{{watch}}" == "--watch" ]]; then \
    cargo watch -w src -x clippy; \
  else \
    cargo clippy; \
  fi

test watch="":
  if [[ "{{watch}}" == "--watch" ]]; then \
    cargo watch -w src -x 'test --locked --frozen --all-features -- --nocapture'; \
  else \
    cargo test --locked --frozen --all-features -- --nocapture; \
  fi

build:
  cargo build

update:
  cargo update
  nix flake update
