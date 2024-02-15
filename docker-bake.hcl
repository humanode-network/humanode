group "default" {
  targets = ["humanode-peer", "robonode-server", "robonode-keygen", "aio"]
}

target "humanode-peer" {
  inherits = ["docker-metadata-action-humanode-peer"]
  dockerfile = "Dockerfile"
  target = "humanode-peer"
}

target "robonode-server" {
  inherits = ["docker-metadata-action-robonode-server"]
  dockerfile = "Dockerfile"
  target = "robonode-server"
}

target "robonode-keygen" {
  inherits = ["docker-metadata-action-robonode-keygen"]
  dockerfile = "Dockerfile"
  target = "robonode-keygen"
}

target "aio" {
  inherits = ["docker-metadata-action-aio"]
  dockerfile = "Dockerfile"
  target = "aio"
}

# Targets to allow injecting customizations from Github Actions.

target "docker-metadata-action-humanode-peer" {}
target "docker-metadata-action-robonode-server" {}
target "docker-metadata-action-robonode-keygen" {}
target "docker-metadata-action-aio" {}
