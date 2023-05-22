import os
import toml

linker = 'kernel/linker.ld'

with open('Cargo.toml', 'r') as file:
    toml_data = toml.loads(file.read())
    bin_sections = toml_data.get('bin', [])
    apps = [section.get('name') for section in bin_sections]
    for app in apps:
        print('processing app "%s"' % app)
        os.system(f'cargo build --bin {app} --release')
        os.system(f'cp ./target/riscv64gc-unknown-none-elf/release/{app} ./prog')
        os.system(f'rust-objcopy --binary-architecture=riscv64 ./prog/{app} --strip-all -O binary ./bin/{app}.bin')
    