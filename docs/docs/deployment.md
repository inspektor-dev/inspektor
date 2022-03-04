---
sidebar_position: 4
title: Deployment
---
## Prerequisites
* Docker 
* Docker-Compose

## Docker
Install Docker by going to the [Docker](https://docs.docker.com/desktop/) website. Once done, install [Docker Compose](https://docs.docker.com/compose/install/) by going to the Docker-Compose website

After the installation process, create a docker-compose file in your project directory. The docker-compose file consists of details of the containers. We are having two containers, one is for the Postgres database and the second is for the control plane.

Here are the details we need to add to our docker-compose file.

```yaml
version: '3.8'

  services:
    postgres:
      container_name: postgres_demo_container
      image: postgres:13.5
      environment:
          POSTGRES_USER: "postgres"
          POSTGRES_PASSWORD: "postgrespass"
          PGDATA: /data/postgres/demo
      volumes:
        - postgres:/data/postgres/demo
      network_mode: host
      restart: unless-stopped 
    controlplane:
      container_name: controlplane
      image: schoolboy/inspektor-controlplane:latest
      volumes:
       - "./config.yaml:/config.yaml"
      depends_on:
        - "postgres"
      network_mode: host
      command: "./wait-for localhost:5432 -- ./inspektor"
  volumes:
    postgres:
```

## The Control Plane

Once Docker is installed, let’s deploy the control plane. You need to create a config.yaml file that contains the following config.

The controlplane requires followinig configuration file to run.

```yaml
#controlplane needs Postgres access to store metadata
postgres_host: "localhost"
postgres_port: "5432"
database_name: "postgres"
postgres_username: "postgres"
postgres_password: "postgrespass"
jwt_key: "demokey"
policy_repo: "https://github.com/poonai/inspektor-policy.git"
```

Run the below command in your terminal to run the control plane.

```
Docker-compose
```
Now that you have everything set up and running, it’s time to put the control plane to use. Go to localhost:3123 and log in using the user name ‘admin’ and password ‘admin’. Use the user menu to create your users.

This is your dashboard.

![](https://inspektor-dev.netlify.app/assets/images/dashboard-f602938bc20e663021a9ff45b062f414.png)

You can tap on add data sources to create a new data source. Copy the secret token of added data source so that the data plane can connect to the control plane.

Use these credentials to deploy the data plane. The process to deploy the data plane is described below

## The Data Plane

Once you are done with Control Plane, let’s attach the data plane to the data source and control plane.
Create a new configuration file dataplane_config.yaml and add this code to it


```yaml
# secret_token obtatined by creating datasource in the dashboard
driver_type: "postgres"
controlplane_addr: "controlplane:5003"
secret_token: "asdfasdfascdasdfasdfasdfasdfasd"  
postgres_config:
  target_addr: "postgres"
  target_port: "5432"
  target_username: "postgres"
  target_password: "postgrespass"
  proxy_listen_port: "8081"
```

Use this command to deploy the data plane

```
docker run -v $(pwd)/dataplane_config.yaml:/dataplane_config.yaml --network=host  schoolboy/inspektor-dataplane:latest ./inspektor --config_file ./dataplane_config.yaml

```
