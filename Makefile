arch ?= x86_64
target ?= $(arch)-micro_os
kernel := build/kernel-$(arch).bin
iso := build/os-$(arch).iso
build_type := debug

linker_script := src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg
assembly_source_files := $(wildcard src/arch/$(arch)/*.asm)
assembly_object_files := $(patsubst src/arch/$(arch)/%.asm, \
	build/arch/$(arch)/%.o, $(assembly_source_files))
rust_source_files := $(shell find src -type f -name "*.rs")
rust_config_file := .cargo/config.toml
rust_target_file := x86_64-micro_os.json

rust_lib := target/$(target)/$(build_type)/libmicro_os.a

rust_os := target/$(target)/$(build_type)/libmicro_os_run.a

test_iso := build/os-$(arch)-test.iso
test_rust_os := target/$(target)/$(build_type)/libmicro_os_test.a
test_kernel := build/kernel-$(arch)-test.bin
qemu := qemu-system-x86_64

.PHONY: all clean run iso test test_iso

all: $(kernel)

run: $(iso)
	$(qemu) \
	-device isa-debug-exit,iobase=0xf4,iosize=0x04 \
	-serial mon:stdio -cdrom $(iso) || \
	{ \
		code=$$?; \
		if [ $$code -ne 0 ] && [ $$code -ne 33 ]; then \
			echo "QEMU: error exited with code $$code"; \
		fi; \
		exit 0; \
	}

clean:
	rm -r build
	cargo clean

iso: $(iso)

$(iso): $(kernel) $(grub_cfg)
	mkdir -p build/isofiles/boot/grub
	cp $(kernel) build/isofiles/boot/kernel.bin
	cp $(grub_cfg) build/isofiles/boot/grub
	grub-mkrescue -d /usr/lib/grub/i386-pc -o $(iso) build/isofiles
	rm -r build/isofiles

$(kernel): $(rust_os) $(assembly_object_files) $(linker_script)
	ld -n -T $(linker_script) -o $(kernel) $(assembly_object_files) $(rust_os)

$(rust_os): $(rust_source_files) $(rust_config_file) $(rust_target_file)
	cargo build -Z json-target-spec
	mv $(rust_lib) $(rust_os)

test: $(test_iso)
	$(qemu) \
	-device isa-debug-exit,iobase=0xf4,iosize=0x04 \
	-serial mon:stdio -cdrom $(test_iso) || \
	{ \
		code=$$?; \
		if [ $$code -ne 0 ] && [ $$code -ne 33 ]; then \
			echo "QEMU: error exited with code $$code"; \
		fi; \
		exit 0; \
	}

test_iso: $(test_iso)

$(test_iso): $(test_kernel) $(grub_cfg)
	mkdir -p build/isofiles/boot/grub
	cp $(test_kernel) build/isofiles/boot/kernel.bin
	cp $(grub_cfg) build/isofiles/boot/grub
	grub-mkrescue -d /usr/lib/grub/i386-pc -o $(test_iso) build/isofiles
	rm -r build/isofiles

$(test_kernel): $(test_rust_os) $(assembly_object_files) $(linker_script)
	ld -n -T $(linker_script) -o $(test_kernel) $(assembly_object_files) $(test_rust_os)

$(test_rust_os): $(rust_source_files) $(rust_config_file) $(rust_target_file)
	cargo build --features use_test -Z json-target-spec
	mv $(rust_lib) $(test_rust_os)

# compile assembly files
build/arch/$(arch)/%.o: src/arch/$(arch)/%.asm
	mkdir -p $(shell dirname $@)
	nasm -felf64 $< -o $@