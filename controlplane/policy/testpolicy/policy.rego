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

default allow = false

default protected_attributes = []

default allowed_attributes = []

role_permission := {"support": [{"postgres-prod": {
	"insert": {"allowed": true, "allowed_attributes": {"postgres.public.kits"}},
	"update": {"allowed": false},
	"copy": {"allowed": true},
	"view": {"allowed": true, "protected_attributes": {"postgres.public.kids", "prod"}},
}}]}

allow {
	permission.allowed
}

allowed_attributes = intersection(attributes) {
	attributes := {attribute | attribute := permission.allowed_attributes}
}

protected_attributes = intersection(attributes) {
	attributes := {attributes | attributes := permission.protected_attributes}
}

permission = resources[_][input.datasource][input.action]

resources[resource] {
	resource = role_permission[input.groups[_]][_]
}