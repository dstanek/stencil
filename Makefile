DOCKER := docker

TEST_IMAGE_NAME:= stencil-tests
TEST_IMAGE_DETAILS := .test-image.json

$(TEST_IMAGE_DETAILS): tests/Dockerfile tests/requirements.txt
	@echo "Building docker image..."
	@$(DOCKER) build -t $(TEST_IMAGE_NAME) tests
	@$(DOCKER) inspect $(TEST_IMAGE_NAME) > $(TEST_IMAGE_DETAILS)

.PHONY: stencil
stencil:
	@echo "Building stencil..."
	@cargo build

.PHONY: tests
tests: $(TEST_IMAGE_DETAILS) stencil
	@echo "Running tests..."
	@$(DOCKER) run --rm \
		-e GITHUB_TOKEN \
		-v $(PWD)/tests/:/tests:Z \
		-v $(PWD)/target/debug/:/stencil:Z \
		$(TEST_IMAGE_NAME)
