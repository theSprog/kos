OS_DIR = target/riscv64gc-unknown-none-elf/release
QEMU_FLAGS = -machine virt -nographic -smp 4
QEMU_BIOS = -bios ./boot/rustsbi-qemu.bin -device loader,file=./kos.bin,addr=0x80200000
QEMU = qemu-system-riscv64
TOOLS = ~/riscv64-elf-tools/bin
KERNEL = kos
KERNEL_BIN = kos.bin

.PHONY: build run debugs debugc clean

build: clean
	@cd ./user && make build && cd ..
	cargo b --release
	ln -sf $(OS_DIR)/$(KERNEL) ./$(KERNEL)
	rust-objcopy --strip-all ./$(KERNEL) -O binary ./$(KERNEL_BIN)

run: build
	$(QEMU) $(QEMU_FLAGS) $(QEMU_BIOS)

# server
debugs: build
	$(QEMU) $(QEMU_FLAGS) $(QEMU_BIOS) -s -S

# client
debugc:
	$(TOOLS)/riscv64-unknown-elf-gdb -ex 'file ./kos' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234' -ex 'b *0x80200000' -ex 'c'

clean:
	@rm -f kos
	@rm -f kos.bin
	@cargo clean
