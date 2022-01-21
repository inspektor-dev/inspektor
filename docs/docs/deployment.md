---
sidebar_position: 4
title: Deployment
---
## Controlplane

Before attaching dataplane to data source, we must run the controlplane. Controlplane uses postgres database to store all the state. So, please have a running postgres database before runnning controlplane.

The controlplane requires followinig configuration file to run.

```yaml
postgres_host: "postgres"
postgres_port: "5432"
database_name: "postgres"
postgres_username: "postgres"
postgres_password: "postgrespass"
jwt_key: "demokey"
policy_repo: "https://github.com/poonai/inspektor-policy.git"
github_access_token: "your github access token if it's private repos"
```

create the config file with the name `config.yaml` with above config parameters.

Use docker to run controlplane

```
 docker run -v $(pwd)/config.yaml:/config.yaml --network=host  schoolboy/inspektor-controlplane:latest ./inspektor
```

## Dataplane

Now that we have running controlplane, let's attach the dataplane to data source and controlplane.

The dataplane requires following configuration file.

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

Use the following command to run dataplane

```
docker run -v $(pwd)/dataplane_config.yaml:/dataplane_config.yaml --network=host  schoolboy/inspektor-dataplane:latest ./inspektor --config_file ./dataplane_config.yaml

```