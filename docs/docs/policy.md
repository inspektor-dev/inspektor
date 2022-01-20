---
sidebar_position: 2
title: Policy
---

Inspektor policy are written using [Open Policy Agent](https://www.openpolicyagent.org/). So, it's advised to learn [rego](https://www.openpolicyagent.org/docs/latest/policy-language/) language to write policies for inspektor.

All the inspektor polices should be defined in github repository. When ever changes happen controlplane pull the lastest policy from the github respository and enforce the lastest policy.

Inspektor query the policy with the following input.

```json
{
    "data_source":"datasourcename",
    "groups":["dev", "support"],
    "db_name":"postgres"
}

```

Inspektor query the following rules.

- allow (denotes that user is allowed to do read operation)
- protected_columns (denotes the list of protected column that should not be exposed to the user)

### Example inpektor policy using open policy agent.

```rego
package inspektor.resource.acl
import future.keywords.in

role_permission := {
	"dev": [{"postgres-prod": {
		"insert": {"inspektor": false},
		"update": {"inspektor": false},
		"protected_fields": {"postgres": {"public.data_sources.side_car_token", "public.customer.email"}},
	}}],
	"admin": [{"postgres-prod": {
		"insert": {"inspektor": false},
		"update": {"inspektor": false},
		"protected_fields": {"postgres": {"public.data_sources.side_car_token"}},
	}}],
}

default allow = false

default protected_columns = []

allow {
	resources[_][input.data_source]
}

protected_columns = intersection(cs) {
	cs := {columns | columns := resources[_][input.data_source].protected_fields[input.db_name]} # builds the set of sets
}

resources[resource] {
	resource = role_permission[input.groups[_]][_]
}
```