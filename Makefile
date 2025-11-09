EXE = Cherry
EVALFILE = networks/default.bin

ifeq ($(OS),Windows_NT)
NAME := $(EXE).exe
else
NAME := $(EXE)
endif

native:
	cargo rustc --release -p cherry -- -C target-cpu=native --emit link=$(NAME)
datagen:
	cargo rustc --release -p cherry --features datagen -- -C target-cpu=native --emit link=$(NAME)