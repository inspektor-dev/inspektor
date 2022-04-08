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
	"html/template"
	"inspektor/apiproto"
	"inspektor/slackbot"
	"inspektor/utils"
	"strings"
	"sync"
	"time"

	"go.uber.org/zap"
)

// MetricsHandler is responsible for sending daily query statics to the
// slack bot.
type MetricsHandler struct {
	sync.Mutex
	slackbot     *slackbot.SlackBot
	groupMetrics map[string]*aggregatedMetrics
}

func NewMetricsHandler(bot *slackbot.SlackBot) *MetricsHandler {
	return &MetricsHandler{
		groupMetrics: map[string]*aggregatedMetrics{},
		slackbot:     bot,
	}
}

// aggregatedMetrics holds the aggregated metrics for each group.
type aggregatedMetrics struct {
	QueryMetrics map[string]*Collection
}

// Collection holds all the metrics related to the collection.
type Collection struct {
	Name       string
	Properties map[string]struct{}
	Count      int
}

// AggregateMetrics will aggregate the metrics.
func (m *MetricsHandler) AggregateMetrics(groups []string, metrics []*apiproto.Metric) {
	m.Lock()
	defer m.Unlock()
	for _, group := range groups {
		groupAggregation, ok := m.groupMetrics[group]
		if !ok {
			groupAggregation = &aggregatedMetrics{
				QueryMetrics: make(map[string]*Collection),
			}
		}
		for _, metric := range metrics {
			collection, ok := groupAggregation.QueryMetrics[metric.CollectionName]
			if !ok {
				collection = &Collection{
					Properties: map[string]struct{}{},
					Name:       metric.CollectionName,
				}
			}
			collection.Count++
			for _, property := range metric.PropertyName {
				collection.Properties[property] = struct{}{}
			}
			groupAggregation.QueryMetrics[metric.CollectionName] = collection
		}
		m.groupMetrics[group] = groupAggregation
	}
}

var Report = `
## Daily Database Activity Report 😎
{{ range $key, $value := .}}
**group: {{$key}}**
### query analytics
| Table Name   | Columns  | Processed Queries   | 
|---|---|---|
{{ range $dbName, $metrics := $value.QueryMetrics}}
|{{$dbName}}|{{join $metrics.Properties }}|{{$metrics.Count}}|
{{end}}
{{end}}
`

func (m *MetricsHandler) Start() {
	timer := m.getReportTicker()
	for {
		<-timer.C
		// reset the timer for the next day
		timer = m.getReportTicker()
		// prepare the report and post it on slack.
		report, err := m.generateReport()
		if err != nil {
			utils.Logger.Error("error while generating analytucs report", zap.String("err", err.Error()))
			continue
		}
		err = m.slackbot.PostMarkdownMsg(report)
		if err != nil {
			utils.Logger.Error("error while publishing daily report", zap.String("err_msg", err.Error()))
		}
	}
}

// generateReport will generate report for the aggregated metrics.
func (m *MetricsHandler) generateReport() (string, error) {
	// add join function since columns name needs to concated with ,
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
		return "", err
	}
	buf := &bytes.Buffer{}
	err = tmpl.Execute(buf, m.groupMetrics)
	if err != nil {
		return "", err
	}
	return buf.String(), nil
}

// getReportTicker return the ticker when the report supposed to published
func (m *MetricsHandler) getReportTicker() *time.Timer {
	// calculate the timer for tomorrow 10'o clock
	y, mo, d := time.Now().Date()
	today := time.Date(y, mo, d, 10, 0, 0, 0, time.Now().Location())
	tommorow := today.Add(14 * time.Hour)
	return time.NewTimer(time.Until(tommorow))
}
