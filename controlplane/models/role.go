package models

import "gorm.io/gorm"

type Role struct {
	gorm.Model
	ObjectID uint
	Name     string
	Type     string
}

const UserType = "USER"

const DataSourceType = "DATA_SOURCE"
