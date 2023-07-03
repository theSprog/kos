import os
import toml
import logging

linker = 'kernel/linker.ld'


def build(logger) :
    with open('Cargo.toml', 'r') as file:
        toml_data = toml.loads(file.read())
        bin_sections = toml_data.get('bin', [])
        apps = [section.get('name') for section in bin_sections]
        for app in apps:
            logger.info('processing app "%s"' % app)
            # print('processing app "%s"' % app)
            os.system(f'cargo build --bin {app} --release')
            os.system(f'cp ./target/riscv64gc-unknown-none-elf/release/{app} ./prog')
            os.system(f'rust-objcopy --binary-architecture=riscv64 ./prog/{app} --strip-all -O binary ./bin/{app}.bin')

if __name__ == '__main__':
    # 配置日志记录器
    logging.basicConfig(
        level=logging.DEBUG,  # 设置日志级别为 DEBUG
        format='%(levelname)s - %(message)s',  # 日志格式
    )
    logger = logging.getLogger()
    build(logger)