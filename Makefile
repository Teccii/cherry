EXE = Cherry
EVALFILE = networks/cherry_768-256-v1.bin

ifeq ($(OS),Windows_NT)
NAME := $(EXE).exe
else
NAME := $(EXE)
endif

native:
	cargo rustc --release -p cherry -- -C target-cpu=native --emit link=$(NAME)
datagen:
	cargo rustc --release -p cherry --features datagen -- -C target-cpu=native --emit link=$(NAME)
tune:
	cargo rustc --release -p cherry --features tune -- -C target-cpu=native --emit link=$(NAME)