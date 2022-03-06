package models

import (
	"encoding/json"
	"inspektor/utils"

	"gorm.io/datatypes"
	"gorm.io/gorm"
)

type User struct {
	gorm.Model
	Name     string         `json:"name"`
	Password string         `json:"-"`
	Meta     datatypes.JSON `json:"-"`
	UserMeta *UserMeta      `gorm:"-" json:"meta"`
	Roles    []string       `gorm:"-" json:"roles"`
}

func (u *User) UnmarshalMeta() {
	meta := &UserMeta{}
	if len(u.Meta) != 0 {
		json.Unmarshal(u.Meta, &meta)
	}
	u.UserMeta = meta
}

func (u *User) MarshalJSON() ([]byte, error) {
	u.UnmarshalMeta()
	return json.Marshal(map[string]interface{}{"username": u.Name, "roles": u.Roles, "meta": u.UserMeta, "id": u.ID})
}

func (u *User) MarshalMeta() {
	buf, err := json.Marshal(&u)
	utils.Check(err)
	u.Meta = buf
}

type UserMeta struct {
	FirstPasswordReset bool `json:"firstPasswordReset"`
}
