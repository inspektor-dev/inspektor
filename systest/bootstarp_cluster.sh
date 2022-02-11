# Copyright 2022 poonai
# 
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
# 
#     http://www.apache.org/licenses/LICENSE-2.0
# 
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

rm controlplane
rm dataplane

cd ../controlplane

go build .

mv inspektor ../systest/controlplane

cd ../

cargo build 

mv target/debug/inspektor systest/dataplane

cd systest

docker-compose up -d

./wait.sh localhost:5432 

echo "postgres started"

psql "sslmode=disable host=localhost port=5432 dbname=postgres user=postgres pass=postgrespass" < seed.sql
