# make [DEBUG=false]
DEBUG ?= true

HEADERS = item.h list.h
TARGET := app

CFLAGS = -Wall -mtune=core2 -falign-functions=64 -gno-record-gcc-switches
ifeq ($(DEBUG), true)
	CFLAGS += -g -O0 -gdwarf-2
	TARGET := $(addsuffix -debug,${TARGET})
else
	CFLAGS += -g -O3 -fno-strict-aliasing -fno-caller-saves
	TARGET := $(addsuffix -release,${TARGET})
endif

default: app

app: main.c item.c $(HEADERS)
	gcc $(CFLAGS) -c item.c -o item.o
	gcc $(CFLAGS) -c main.c -o main.o
	gcc item.o main.o -o ${TARGET}

clean:
	-rm -f item.o
	-rm -f main.o
	-rm -f app





