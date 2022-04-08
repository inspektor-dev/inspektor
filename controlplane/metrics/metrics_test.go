// Copyright 2022 poonai
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package metrics

import (
	"bytes"
	"fmt"
	"html/template"
	"strings"
	"testing"
)

func TestReportGeneration(t *testing.T) {
	tmpl, err := template.New("report").Funcs(template.FuncMap{
		"join": func(val map[string]struct{}) string {
			arr := []string{}
			for key := range val {
				arr = append(arr, key)
			}
			return strings.Join(arr, ",")
		},
	}).Parse(Report)
	if err != nil {
		t.Fatal(err)
	}
	data := make(map[string]*aggregatedMetrics)
	data["dev"] = &aggregatedMetrics{
		QueryMetrics: map[string]*Collection{"user": &Collection{Name: "user",
			Properties: map[string]struct{}{"age": struct{}{}, "name": struct{}{}},
			Count:      19,
		}},
	}
	buf := &bytes.Buffer{}
	err = tmpl.Execute(buf, data)
	if err != nil {
		t.Fatal(err)
	}
	fmt.Println(buf.String())
}
