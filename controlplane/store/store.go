package store

import (
	"inspektor/models"
	"inspektor/types"
	"inspektor/utils"

	"go.uber.org/zap"
	"golang.org/x/crypto/bcrypt"
	"gorm.io/gorm"
)

type Store struct {
	db *gorm.DB
}

func NewStore(db *gorm.DB) (*Store, error) {
	s := &Store{
		db: db,
	}
	return s, s.init()
}

// init will seed the admin user if the admin account doesn't exist.
func (s *Store) init() error {
	// check whether admin account exist.
	var count int64
	err := s.db.Model(&models.User{}).Where("user_name = ?", "admin").Count(&count).Error
	if err != nil {
		utils.Logger.Error("error while retirving admin account", zap.String("err_msg", err.Error()))
		return err
	}
	if count != 0 {
		return nil
	}
	password, err := bcrypt.GenerateFromPassword([]byte("admin"), bcrypt.DefaultCost)
	if err != nil {
		utils.Logger.Error("error while hashing password", zap.String("err_msg", err.Error()))
		return err
	}
	// admin account is not present so let's create an admin account.
	user := &models.User{
		Name:     "admin",
		Password: string(password),
	}
	user.UserMeta = &models.UserMeta{
		FirstPasswordReset: false,
	}
	user.MarshalMeta()
	if err := s.db.Create(user).Error; err != nil {
		utils.Logger.Error("error while creating admin account", zap.String("err_msg", err.Error()))
		return err
	}
	if err := s.WriteRoleForObjectID(user.ID, "admin"); err != nil && err != types.ErrRoleAlreadyExist {
		utils.Logger.Error("error while creating role for the default admin user", zap.String("err_msg", err.Error()))
	}
	return nil
}

func (s *Store) GetUserByName(name string) (*models.User, error) {
	user := &models.User{}
	err := s.db.Model(&models.User{}).Where("name = ?", name).First(user).Error
	return user, err
}

func (s *Store) WriteRoleForObjectID(id uint, name string) error {
	// if the role already exist for the object then we should throw error.
	// TODO: simple way is that we can put primary key constraint on two columns.

	// check role exist for the the object id.
	var count int64
	if err := s.db.Model(&models.Role{}).Where("object_id = ?", id).Count(&count).Error; err != nil {
		utils.Logger.Error("error while checking whether role exist for the given object id", zap.Uint("object_id", id))
		return err
	}
	if count > 0 {
		return types.ErrRoleAlreadyExist
	}
	role := &models.Role{
		ObjectID: id,
		Name:     name,
	}
	return s.db.Create(role).Error
}

func (s *Store) GetRolesForObjectID(id uint) ([]string, error) {
	roles := []*models.Role{}
	if err := s.db.Model(&models.Role{}).Where("object_id = ?", id).First(&roles).Error; err != nil {
		utils.Logger.Error("error while retriving roles for the object", zap.Uint("object_id", id))
		return []string{}, err
	}
	out := make([]string, len(roles))
	for _, role := range roles {
		out = append(out, role.Name)
	}
	return out, nil
}
