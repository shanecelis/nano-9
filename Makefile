GOLDEN_DIR := tests/golden
CARTS := $(wildcard $(GOLDEN_DIR)/*.p8)
EXPECTED := $(patsubst $(GOLDEN_DIR)/%.p8,$(GOLDEN_DIR)/%-expected.png,$(CARTS))

.PHONY: golden
golden: $(EXPECTED)

$(GOLDEN_DIR)/%-expected.png: $(GOLDEN_DIR)/%.p8
	bin/golden-pico8 $< -o $@
