package inspektor.resource_acl

role_permission := {
        "dev":[
                {
                        "postgres_production":{
                                "read": {"rows": 1},
                                "update": {"rows": 1},
                                "protected_fields":[
                                                                        "user.pan",
                                                                   ]
                        }
                }
        ]
}
default allow = false

permission = role_permission[input.group]
rules = permission[_][input.resource]
rule = rules[input.action]
allow {
    rule
}
rows = rule["rows"]