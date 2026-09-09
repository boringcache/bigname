variable "RUST_VERSION" {
  default = "1.93.1"
}

variable "IMAGE_NAME" {
  default = "bigname"
}

variable "SOURCE_SHA" {
  default = "unknown"
}

variable "SOURCE_URL" {
  default = "https://github.com/ensdomains/bigname"
}

variable "PUBLISH_IMAGE" {
  default = false
}

target "default" {
  context = "."
  dockerfile = "Dockerfile"
  platforms = ["linux/amd64"]
  args = {
    RUST_VERSION = RUST_VERSION
    BIGNAME_BUILD_SHA = SOURCE_SHA
  }
  tags = ["${IMAGE_NAME}:${substr(SOURCE_SHA, 0, 7)}", "${IMAGE_NAME}:latest"]
  labels = {
    "org.opencontainers.image.source" = SOURCE_URL
    "org.opencontainers.image.revision" = SOURCE_SHA
  }
  output = PUBLISH_IMAGE ? ["type=registry"] : ["type=docker"]
}
