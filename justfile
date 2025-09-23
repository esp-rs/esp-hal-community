all: check-risc-v build-risc-v check-xtensa build-xtensa clippy

clippy:
  cargo clippy --features "esp32c6,esp-hal/unstable" --release

check-risc-v: build-esp32c3 build-esp32c6 build-esp32h2

check-esp32c3:
  cargo check --features "esp32c3,esp-hal/unstable" --target=riscv32imc-unknown-none-elf --release
check-esp32c6:
  cargo check --features "esp32c6,esp-hal/unstable" --target=riscv32imac-unknown-none-elf --release
check-esp32h2:
  cargo check --features "esp32h2,esp-hal/unstable" --target=riscv32imac-unknown-none-elf --release

build-risc-v: build-esp32c3 build-esp32c6 build-esp32h2

build-esp32c3:
  cargo +stable build --examples --features "esp32c3,esp-hal/unstable" --target=riscv32imc-unknown-none-elf --release
build-esp32c6:
  cargo +stable build --examples --features "esp32c6,esp-hal/unstable" --target=riscv32imac-unknown-none-elf --release
build-esp32h2:
  cargo +stable build --examples --features "esp32h2,esp-hal/unstable" --target=riscv32imac-unknown-none-elf --release


check-xtensa:  check-esp32s2 check-esp32s3 check-esp32

check-esp32:
  cargo +esp check --features "esp32,esp-hal/unstable" --target=xtensa-esp32-none-elf --release
check-esp32s2:
  cargo +esp check --features "esp32s2,esp-hal/unstable" --target=xtensa-esp32s2-none-elf --release
check-esp32s3:
  cargo +esp check --features "esp32s3,esp-hal/unstable" --target=xtensa-esp32s3-none-elf --release

build-xtensa:  build-esp32s2 build-esp32s3 build-esp32

build-esp32:
  cargo +esp build --examples --features "esp32,esp-hal/unstable" --target=xtensa-esp32-none-elf --release
build-esp32s2:
  cargo +esp build --examples --features "esp32s2,esp-hal/unstable" --target=xtensa-esp32s2-none-elf --release
build-esp32s3:
  cargo +esp build --examples --features "esp32s3,esp-hal/unstable" --target=xtensa-esp32s3-none-elf --release

run-esp32c3 example:
  cargo +stable run --example {{example}} --features "esp32c3,esp-hal/unstable" --target=riscv32imc-unknown-none-elf --release
run-esp32c6 example:
  cargo +stable run --example {{example}} --features "esp32c6,esp-hal/unstable" --target=riscv32imac-unknown-none-elf --release
run-esp32h2 example:
  cargo +stable run --example {{example}} --features "esp32h2,esp-hal/unstable" --target=riscv32imac-unknown-none-elf --release
run-esp32 example:
  cargo +esp run --example {{example}} --features "esp32,esp-hal/unstable" --target=xtensa-esp32-none-elf --release
run-esp32s2 example:
  cargo +esp run --example {{example}} --features "esp32s2,esp-hal/unstable" --target=xtensa-esp32s2-none-elf --release
run-esp32s3 example:
  cargo +esp run --example {{example}} --features "esp32s3,esp-hal/unstable" --target=xtensa-esp32s3-none-elf --release

msrv-buzzer:
  cargo msrv find --target riscv32imac-unknown-none-elf -- cargo check -p esp-hal-buzzer --features "esp32c6,esp-hal/unstable" --target riscv32imac-unknown-none-elf
msrv-smartled:
  cargo msrv find --target riscv32imac-unknown-none-elf -- cargo check -p esp-hal-smartled --features "esp32c6,esp-hal/unstable" --target riscv32imac-unknown-none-elf
