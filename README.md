# Distrust

## Quickstart (dev)
```console
cargo r --release
```

## Containerized deploy
```console
podman build -t distrust .
podman run -d --name distrust -p 6969:6969 -v ./data:/app/data:Z distrust
```
