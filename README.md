# ckb-contract-minitest
## Usage
Prepare the binary sudt contract file
```bash
cd contracts/sudt
rustup target add riscv64imac-unknown-none-elf
cargo build --release --target riscv64imac-unknown-none-elf
cp -f target/riscv64imac-unknown-none-elf/release/sudt ../../build/release/sudt
cd ../..
```

Run test

`cargo test --test sudt`

