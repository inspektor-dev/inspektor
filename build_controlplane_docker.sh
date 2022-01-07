cd controlplane
go build .

cd ..

docker build .  -t schoolboy/inspektor-controlplane -f Dockerfile.controlplane