variable "RUST_VERSION" {
  default = "1.93.1"
}

variable "SOURCE_SHA" {
  default = "unknown"
}

target "default" {
  context = "."
  dockerfile = "Dockerfile"
  platforms = ["linux/amd64"]
  args = {
    RUST_VERSION = RUST_VERSION
    BIGNAME_BUILD_SHA = SOURCE_SHA
  }
  tags = ["bigname-api-shutdown:${SOURCE_SHA}"]
  labels = {
    "org.opencontainers.image.revision" = SOURCE_SHA
  }
  output = ["type=docker"]
}
