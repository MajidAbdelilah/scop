SRCS=  src/main.rs src/obj_parcer.rs
CFLAGS= --release
CPP= cargo
NAME= scop

.PHONY: clean

all: ./target/release/$(NAME)

./target/release/$(NAME): $(SRCS)
	$(CPP) build $(CFLAGS)

clean:
	rm -rf target/release/build

fclean: clean
	rm -rf ./target/release/$(NAME)

re: fclean all