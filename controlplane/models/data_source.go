package models

import (
	"time"

	"gorm.io/gorm"
)

type DataSource struct {
	ID              uint           `gorm:"primarykey" json:"id"`
	CreatedAt       time.Time      `json:"-"`
	UpdatedAt       time.Time      `json:"-"`
	DeletedAt       gorm.DeletedAt `gorm:"index"`
	Name            string         `gorm:"unique" json:"name"`
	Type            string         `json:"type"`
	SideCarHostName string         `json:"sidecarHostname"`
	SideCarToken    string         `json:"sidecarToken,omitempty"`
}
