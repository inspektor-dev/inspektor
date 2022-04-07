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
	"inspektor/apiproto"
	"inspektor/slackbot"
	"sync"
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
	queryMetrics map[string]*Collection
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
				queryMetrics: make(map[string]*Collection),
			}
		}
		for _, metric := range metrics {
			collection, ok := groupAggregation.queryMetrics[metric.CollectionName]
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
			groupAggregation.queryMetrics[metric.CollectionName] = collection
		}
		m.groupMetrics[group] = groupAggregation
	}
}
