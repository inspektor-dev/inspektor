#  Copyright 2021 Balaji (rbalajis25@gmail.com)
#  
#  Licensed under the Apache License, Version 2.0 (the "License");
#  you may not use this file except in compliance with the License.
#  You may obtain a copy of the License at
#  
#  http://www.apache.org/licenses/LICENSE-2.0
#  
#  Unless required by applicable law or agreed to in writing, software
#  distributed under the License is distributed on an "AS IS" BASIS,
#  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#  See the License for the specific language governing permissions and
#  limitations under the License.

package inspektor.resource.acl

import future.keywords.in

role_permission := {
	"dev": [{"postgres-prod": {
		"insert": {"inspektor": false},
		"update": {"inspektor": false},
		"protected_fields": {"inspektor": {"public.data_sources.side_car_token"}},
	}}],
	"admin": [{"postgres-prod": {
		"insert": {"inspektor": false},
		"update": {"inspektor": true},
		"protected_fields": {"inspektor": {"public.data_sources.side_car_token"}},
	}}],
}

default allow = false

default protected_columns = []

default insert = false

default update = false

allow {
	resources[_][input.data_source]
}

protected_columns = intersection(cs) {
	cs := {columns | columns := resources[_][input.data_source].protected_fields[input.db_name]} # builds the set of sets
}

insert {
	true in [aggs | aggs := resources[_][input.data_source].insert[input.db_name]]
}

update {
	true in [updates | updates := resources[_][input.data_source].update[input.db_name]]
}

resources[resource] {
	resource = role_permission[input.groups[_]][_]
}
