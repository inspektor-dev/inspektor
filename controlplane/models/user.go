package models

import (
	"encoding/json"
	"inspektor/utils"

	"gorm.io/datatypes"
	"gorm.io/gorm"
)

type User struct {
	gorm.Model
	Name     string
	Password string
	Meta     datatypes.JSON
	UserMeta *UserMeta `gorm:"-"`
}

func (u *User) UnmarshalMeta() {
	meta := &UserMeta{}
	if len(u.Meta) != 0 {
		json.Unmarshal(u.Meta, &meta)
	}
	u.UserMeta = meta
}

func (u *User) MarshalMeta() {
	buf, err := json.Marshal(&u)
	utils.Check(err)
	u.Meta = buf
}

type UserMeta struct {
	FirstPasswordReset bool `json:"firstPasswordReset"`
}