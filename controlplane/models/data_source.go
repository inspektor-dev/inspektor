package models

import (
	"time"

	"gorm.io/gorm"
)

type DataSource struct {
	ID              uint           `gorm:"primarykey" json:"-"`
	CreatedAt       time.Time      `json:"-"`
	UpdatedAt       time.Time      `json:"-"`
	DeletedAt       gorm.DeletedAt `gorm:"index"`
	Name            string         `gorm:"unique"`
	Type            string         `json:"-"`
	SideCarHostName string         `json:"sidecardHostName"`
}
