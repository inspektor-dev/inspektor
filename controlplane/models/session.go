package models

import (
	"encoding/json"
	"inspektor/utils"

	"gorm.io/datatypes"
	"gorm.io/gorm"
)

type Session struct {
	gorm.Model
	ObjectID    uint           `json:"objectID"`
	UserID      uint           `json:"-"`
	Meta        datatypes.JSON `json:"meta"`
	SessionMeta *SessionMeta   `gorm:"-" json:"-"`
	Datasource  *DataSource    `gorm:"-" json:"datasource"`
}

type SessionMeta struct {
	Type             string          `json:"type"`
	PostgresPassword string          `json:"postgresPassword"`
	PostgresUsername string          `json:"postgresUsername"`
	TempRoles        []string        `json:"tempRoles"`
	ExpiresAt        int64           `json:"expiresAt"`
	Context          json.RawMessage `json:"context"`
	TempCreatedBy    string          `json:"tempCreatedBy"`
	Passthrough      bool            `json:"passthrough"`
}

func (s *Session) UnmarshalMeta() {
	meta := &SessionMeta{}
	if len(s.Meta) != 0 {
		json.Unmarshal(s.Meta, &meta)
	}
	s.SessionMeta = meta
}

func (s *Session) MarshalMeta() {
	buf, err := json.Marshal(s.SessionMeta)
	utils.Check(err)
	s.Meta = buf
}
