---
sidebar_position: 3
title: Features
---


## Inspektor basic features

The installation steps will run a Postgres instance, controlplane and dataplane. The dataplane is connected to the same Postgres container that is used by controlplane to store metadata to ease out the complexitiy. But, you can connect to any Postgres instance as you like by tinkering with the dataplane config file.

In this sample database, we want to protect first_name of the actor table. For this, we have to define policy using Open Policy Agent. We have already defined that policy on [sample repository here.](https://github.com/poonai/inspektor-policy.git)


Now hit the [http://localhost:3123](http://localhost:3123) and use the following credentials to login into the dashboard.

```
username: admin
password: admin
```

After login, you'll see the list of datasources that controlplane manages. 

![Dashboard](../static/img/dashboard.png)

The Postgres instance that we want to enforce policy is already added as datasource, now you can click on create credentials button to get login information to access the datasource.

After creating the credentials you'll get to see show credentials button. After clicking you'll get a modal showing the credentials to access the Postgres instance.

![Credentials Modal](../static/img/credentials.png)

Now just use psql to login to the Postgres instance using the copied credentials from the dashboard.

```
psql "sslmode=disable host=localhost port=8081 dbname=postgres user=<username>"
```

After executing the above command, psql will prompt you to enter password. Enter the password
which you copied from the modal to login.

Now that, you logged in. execute a simple select query on the actor table.

```sql
select * from actor;
```

You'll get output similar to this.
```
 actor_id | first_name |  last_name   |      last_update       
----------+------------+--------------+------------------------
        1 |            | GUINESS      | 2020-02-15 09:34:33+00
        2 |            | WAHLBERG     | 2020-02-15 09:34:33+00
        3 |            | CHASE        | 2020-02-15 09:34:33+00
        4 |            | DAVIS        | 2020-02-15 09:34:33+00
        5 |            | LOLLOBRIGIDA | 2020-02-15 09:34:33+00
        6 |            | NICHOLSON    | 2020-02-15 09:34:33+00
        7 |            | MOSTEL       | 2020-02-15 09:34:33+00
        8 |            | JOHANSSON    | 2020-02-15 09:34:33+00
        9 |            | SWANK        | 2020-02-15 09:34:33+00
       10 |            | GABLE        | 2020-02-15 09:34:33+00
       11 |            | CAGE         | 2020-02-15 09:34:33+00
       12 |            | BERRY        | 2020-02-15 09:34:33+00
       13 |            | WOOD         | 2020-02-15 09:34:33+00
       14 |            | BERGEN       | 2020-02-15 09:34:33+00
       15 |            | OLIVIER      | 2020-02-15 09:34:33+00
       16 |            | COSTNER      | 2020-02-15 09:34:33+00
       17 |            | VOIGHT       | 2020-02-15 09:34:33+00
       18 |            | TORN         | 2020-02-15 09:34:33+00
       19 |            | FAWCETT      | 2020-02-15 09:34:33+00
       20 |            | TRACY        | 2020-02-15 09:34:33+00
       21 |            | PALTROW      | 2020-02-15 09:34:33+00

```

You can clearly see that first_name has been hidden from the user by inspektor. Now, get your hands dirty by forking inspektor demo policy repo and play with inspektor. Probably, you can run an insert statement. 

We are active on discord, so if you need any help please do reach out to us. We are more than happy to help you. Here is the discord invite link:  https://discord.com/invite/YxZbDJHTxf.

See you soon. Can't wait to see what cool things you can do with Inspektor. 