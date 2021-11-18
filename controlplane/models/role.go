package models

import "gorm.io/gorm"

type Role struct {
	gorm.Model
	ObjectID uint
	Name     string
}
