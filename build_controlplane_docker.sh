cd controlplane

go build .

cd dashboard

npm run build

cd ../..


docker build .  -t  schoolboy/inspektor-controlplane:latest -f Dockerfile.controlplane --no-cache