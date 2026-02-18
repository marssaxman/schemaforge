# project-specific settings
EXECNAME:=schemaforge
CCFLAGS:=-Werror -Wall -g
LDFLAGS:=-lstdc++

# boilerplate rules
SOURCES:=$(shell find src -name *.c -o -name *.cpp)
OBJECTS:=$(addsuffix .o,$(basename $(patsubst src/%,build/%,$(SOURCES))))
TARGET:=build/$(EXECNAME)
default: $(TARGET)
$(TARGET): $(OBJECTS)
	@mkdir -p $(@D)
	$(CC) -o $@ $^ $(LDFLAGS)
build/%.o: src/%.cpp
	@mkdir -p $(@D)
	$(CC) -std=c++17 $(CCFLAGS) -Isrc -MD -MP -c $< -o $@
build/%.o: src/%.c
	@mkdir -p $(@D)
	$(CC) -std=c17 $(CCFLAGS) -Isrc -MD -MP -c $< -o $@
clean:
	-rm -rf build
install:
	cp $(TARGET) /usr/bin/$(EXECNAME)
format:
	clang-format -i $(shell find src -name *.c -o -name *.cpp -o -name *.h)
.PHONY: clean install format
-include $(shell find build -name *.d)

TESTSRCS:=$(shell find test -name *.cpp)
TESTBINS:=$(basename $(patsubst test/%,build/test/%,$(TESTSRCS)))
build/test/%: test/%.cpp
	@mkdir -p $(@D)
	$(CC) -std=c++17 $(CCFLAGS) -Itest $< -o $@ $(LDFLAGS)

test: $(OBJECTS) $(TESTBINS)
	for test in $(TESTBINS); do $$test; done

