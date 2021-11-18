package models

import "gorm.io/gorm"

type DataSource struct {
	gorm.Model
	Name string `gorm:"unique"`
	Type string
}
