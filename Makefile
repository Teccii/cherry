EXE = Cherry
EVALFILE = networks/default.bin

ifeq ($(OS),Windows_NT)
NAME := $(EXE).exe
else
NAME := $(EXE)
endif

native:
	RUSTFLAGS="-C target-cpu=native" cargo rustc --release -p cherry -- --emit link=$(NAME)
datagen:
	RUSTFLAGS="-C target-cpu=native" cargo rustc --release -p cherry --features datagen -- --emit link=$(NAME)