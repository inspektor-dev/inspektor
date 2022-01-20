---
sidebar_position: 1
title: Inspektor Tutorial
---

## Introduction

Inpektor helps you to enfore access control for all your data sources. 

In this tutorial, you'll be downloading and running the sample docker-compose setup and configure it to enforce access policy on postgres database for different user group. 

## Prerequisite
 - docker
 - docker-compose
 - git
 - psql

## Demo Inspektor setup.

Clone the example inspektor setup in your local machine.

```sh
git clone https://github.com/poonai/inspektor-demo
```

The cloned repository contains neccessary files to setup a demo inspektor environment.

change your current working directory to the cloned respository.

```sh
cd inspektor-demo
```

Run the docker compose file to setup the inspektor demo setup.

```
docker-compose up
```

The setup won't be complete on the first run, because we have to wait till our database is up and ready with the sample data. So, wait till you see the following output.

```
postgres_demo_container | 2022-01-19 10:09:23.152 UTC [49] LOG:  received fast shutdown request
postgres_demo_container | waiting for server to shut down....2022-01-19 10:09:23.158 UTC [49] LOG:  aborting any active transactions
postgres_demo_container | 2022-01-19 10:09:23.159 UTC [49] LOG:  background worker "logical replication launcher" (PID 56) exited with exit code 1
postgres_demo_container | 2022-01-19 10:09:23.159 UTC [51] LOG:  shutting down
postgres_demo_container | 2022-01-19 10:09:23.244 UTC [49] LOG:  database system is shut down
postgres_demo_container |  done
postgres_demo_container | server stopped
postgres_demo_container | 
postgres_demo_container | PostgreSQL init process complete; ready for start up.
postgres_demo_container | 
postgres_demo_container | 2022-01-19 10:09:23.283 UTC [1] LOG:  starting PostgreSQL 13.5 (Debian 13.5-1.pgdg110+1) on x86_64-pc-linux-gnu, compiled by gcc (Debian 10.2.1-6) 10.2.1 20210110, 64-bit
postgres_demo_container | 2022-01-19 10:09:23.283 UTC [1] LOG:  listening on IPv4 address "0.0.0.0", port 5432
postgres_demo_container | 2022-01-19 10:09:23.283 UTC [1] LOG:  listening on IPv6 address "::", port 5432
postgres_demo_container | 2022-01-19 10:09:23.296 UTC [1] LOG:  listening on Unix socket "/var/run/postgresql/.s.PGSQL.5432"
postgres_demo_container | 2022-01-19 10:09:23.310 UTC [64] LOG:  database system was shut down at 2022-01-19 10:09:23 UTC
postgres_demo_container | 2022-01-19 10:09:23.380 UTC [1] LOG:  database system is ready to accept connections

```

Please `CTRL+C` after database is initialized.

Now, run the `docker-compose up` again to start the setup again

```sh
docker-compose up
```

We are running `docker-compose up` twice because, I don't know how to make the other service to wait till the database is seeded with the sample data  😂  😂. If you know how to do that, please reach out to me. I really want to fix this ugly part of the tutorial.

## Inspektor basic features

The above steps will run a postgres instance, dataplane and controlplane. The postgres instance is connected to the dataplane. So, here after all the access to the postgres goes through the inpektor dataplane.

The sample database contains sample data taken from the following repository.
- https://github.com/devrimgunduz/pagila

In this sample database, we want to protect email column of customer table. For this we have to define policy using Open Policy Agent. Same policy is already defined in the demo repo. 

To know more about how to define policy, please refer policy section of the docs.

Now hit the [http://localhost:3123](http://localhost:3123) and use the following credentials to login to the dashboard.

```
username: admin
password: admin
```

After login, you'll see the list of datasources that controlplane manages. 

![Dashboard](../static/img/dashboard.png)

The postgres instance that we want to enforce policy is already added as datasource, now you can click on create credentials button to get login information to access the datasource.

After creating the credentials you'll get to see show credentials button. After clicking you'll get modal showing the credentials to access the postgres instance.

![Credentials Modal](../static/img/credentials.png)

Now just use psql to login to the postgres instance.

```
psql "sslmode=disable host=localhost port=8081 dbname=postgres user=<username>"
```

After executing the above command, psql will prompt you to enter password. Enter the password
which you copied from the modal to login.

Now that, you logged in. execute a simple select query on customer table.

```sql
select * from customer;
```

You'll get output similar to this.
```
 customer_id | store_id | first_name  |  last_name   |                  email                   | address_id | activebool | create_date |      last_update       | active 
-------------+----------+-------------+--------------+------------------------------------------+------------+------------+-------------+------------------------+--------
           1 |        1 | MARY        | SMITH        | MARY.SMITH@sakilacustomer.org            |          5 | t          | 2020-02-14  | 2020-02-15 09:57:20+00 |      1
           2 |        1 | PATRICIA    | JOHNSON      | PATRICIA.JOHNSON@sakilacustomer.org      |          6 | t          | 2020-02-14  | 2020-02-15 09:57:20+00 |      1
           3 |        1 | LINDA       | WILLIAMS     | LINDA.WILLIAMS@sakilacustomer.org        |          7 | t          | 2020-02-14  | 2020-02-15 09:57:20+00 |      1
           4 |        2 | BARBARA     | JONES        | BARBARA.JONES@sakilacustomer.org         |          8 | t          | 2020-02-14  | 2020-02-15 09:57:20+00 |      1
           5 |        1 | ELIZABETH   | BROWN        | ELIZABETH.BROWN@sakilacustomer.org       |          9 | t          | 2020-02-14  | 2020-02-15 09:57:20+00 |      1
           6 |        2 | JENNIFER    | DAVIS        | JENNIFER.DAVIS@sakilacustomer.org        |         10 | t          | 2020-02-14  | 2020-02-15 09:57:20+00 |      1
           7 |        1 | MARIA       | MILLER       | MARIA.MILLER@sakilacustomer.org          |         11 | t          | 2020-02-14  | 2020-02-15 09:57:20+00 |      1
           8 |        2 | SUSAN       | WILSON       | SUSAN.WILSON@sakilacustomer.org          |         12 | t          | 2020-02-14  | 2020-02-15 09:57:20+00 |      1

```

Admin role doesn't have any protected columns so, email address of the customer table are exposed.


Let's see what happens if we login to the postgres instance using dev role. 

The demo setup already have a user with dev role. You can login to the dashboard using following credentials to obtain the postgres credentials for the `dev@company.io` user.

```
username: dev@company.io
password: hello123
```

If you query the customer table, you won't be seeing email address because our policy tells not to expose email id for the dev role.


```
 customer_id | store_id | first_name  |  last_name   | customer.email | address_id | activebool | create_date |      last_update       | active 
-------------+----------+-------------+--------------+----------------+------------+------------+-------------+------------------------+--------
           1 |        1 | MARY        | SMITH        |                |          5 | t          | 2020-02-14  | 2020-02-15 09:57:20+00 |      1
           2 |        1 | PATRICIA    | JOHNSON      |                |          6 | t          | 2020-02-14  | 2020-02-15 09:57:20+00 |      1
           3 |        1 | LINDA       | WILLIAMS     |                |          7 | t          | 2020-02-14  | 2020-02-15 09:57:20+00 |      1
           4 |        2 | BARBARA     | JONES        |                |          8 | t          | 2020-02-14  | 2020-02-15 09:57:20+00 |      1
           5 |        1 | ELIZABETH   | BROWN        |                |          9 | t          | 2020-02-14  | 2020-02-15 09:57:20+00 |      1
           6 |        2 | JENNIFER    | DAVIS        |                |         10 | t          | 2020-02-14  | 2020-02-15 09:57:20+00 |      1
           7 |        1 | MARIA       | MILLER       |                |         11 | t          | 2020-02-14  | 2020-02-15 09:57:20+00 |      1
           8 |        2 | SUSAN       | WILSON       |                |         12 | t          | 2020-02-14  | 2020-02-15 09:57:20+00 |      1

```
