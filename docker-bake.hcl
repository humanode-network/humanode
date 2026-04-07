variable "TARGETS" {
  type = list(string)
  default = ["humanode-peer", "robonode-server", "robonode-keygen"]
}

group "default" {
  targets = TARGETS
}

target "base" {
  dockerfile = "Dockerfile"
  # ssh = ["default"]
}

target "main" {
  matrix = {
    tgt = TARGETS
  }
  name = tgt
  inherits = ["base", "docker-metadata-action-${tgt}"]
  target = tgt
}

# Targets to allow injecting customizations from Github Actions.

target "docker-metadata-action" {
  matrix = {
    tgt = TARGETS
  }
  name = "docker-metadata-action-${tgt}"
}
