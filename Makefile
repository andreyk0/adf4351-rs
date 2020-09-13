BUILD?=debug
ELF_TARGET:=target/thumbv7em-none-eabihf/$(BUILD)/examples/freq

build:
	cargo build --example freq $(if $(findstring release,$(BUILD)),--release,)

# Requires openocd running
debug: build
	arm-none-eabi-gdb -x openocd.gdb -q $(ELF_TARGET)

doc:
	cargo doc --open

clean:
	cargo clean

.PHONY: \
	build \
	clean \
	debug \
	doc \
