cargo build --release

docker build . -t schoolboy/inspektor-dataplane:latest1 --build-arg CACHEBUST=$(date +%s) --no-cache