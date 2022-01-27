package store

import (
	"errors"
	"inspektor/models"
	"inspektor/types"
	"inspektor/utils"
	"time"

	"github.com/goombaio/namegenerator"
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
	err := s.db.Model(&models.User{}).Where("name = ?", "admin").Count(&count).Error
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
	if err := s.WriteRoleForUserObjectID(user.ID, []string{"admin"}); err != nil && err != types.ErrRoleAlreadyExist {
		utils.Logger.Error("error while creating role for the default admin user", zap.String("err_msg", err.Error()))
	}
	return nil
}

func (s *Store) GetUserByName(name string) (*models.User, error) {
	user := &models.User{}
	err := s.db.Model(&models.User{}).Where("name = ?", name).First(user).Error
	return user, err
}

func (s *Store) WriteRoleForUserObjectID(id uint, roles []string) error {
	// if the role already exist for the object then we should throw error.
	// TODO: simple way is that we can put primary key constraint on two columns.

	// // check role exist for the the object id.
	// var count int64
	// if err := s.db.Model(&models.Role{}).Where("object_id = ? and type = ?", id, models.UserType).Count(&count).Error; err != nil {
	// 	utils.Logger.Error("error while checking whether role exist for the given object id", zap.Uint("object_id", id))
	// 	return err
	// }
	// if count > 0 {
	// 	return types.ErrRoleAlreadyExist
	// }
	rolesObj := []*models.Role{}
	dupmap := map[string]interface{}{}
	for _, role := range roles {
		_, ok := dupmap[role]
		if ok {
			continue
		}
		rolesObj = append(rolesObj, &models.Role{
			ObjectID: id,
			Type:     models.UserType,
			Name:     role,
		})
		dupmap[role] = struct{}{}
	}
	return s.db.Model(&models.Role{}).Create(&rolesObj).Error
}

func (s *Store) GetRolesForObjectID(id uint, objectType string) ([]string, error) {
	roles := []*models.Role{}
	if err := s.db.Model(&models.Role{}).Where("object_id = ? AND type = ?", id, objectType).Find(&roles).Error; err != nil {
		utils.Logger.Error("error while retriving roles for the object", zap.Uint("object_id", id))
		return []string{}, err
	}
	out := make([]string, 0, len(roles))
	for _, role := range roles {
		out = append(out, role.Name)
	}
	return out, nil
}

func (s *Store) GetDataSources(ids ...uint) ([]*models.DataSource, error) {
	dataSources := []*models.DataSource{}
	if err := s.db.Model(&models.DataSource{}).Where("id in (?)", ids).Find(&dataSources).Error; err != nil {
		utils.Logger.Error("error while fetching data sources", zap.String("err_msg", err.Error()))
		return dataSources, err
	}
	return dataSources, nil
}

func (s *Store) GetDatasourceByWhere(query interface{}, args ...interface{}) (*models.DataSource, error) {
	dataSource := &models.DataSource{}
	if err := s.db.Model(&models.DataSource{}).Where(query, args...).First(dataSource).Error; err != nil {
		utils.Logger.Error("error while fetching data source", zap.String("err_msg", err.Error()))
		return nil, err
	}
	return dataSource, nil
}

func (s *Store) GetObjectIDsForRoles(objectType string, roles []string) ([]uint, error) {
	filteredRoles := []*models.Role{}
	if err := s.db.Model(&models.Role{}).Where("type = ? AND name IN (?)", objectType, roles).Find(&filteredRoles).Error; err != nil {
		utils.Logger.Error("error while retriving roles", zap.String("type", objectType), zap.Strings("roles", roles))
		return []uint{}, err
	}
	ids := []uint{}
	for _, role := range filteredRoles {
		ids = append(ids, role.ObjectID)
	}
	return ids, nil
}

func (s *Store) CreateDataSource(datasource *models.DataSource, roles []string) error {
	return s.db.Transaction(func(tx *gorm.DB) error {
		// check any data source exist with that name.
		var count int64
		if err := tx.Model(&models.DataSource{}).Where("name = ?", datasource.Name).Count(&count).Error; err != nil {
			return err
		}
		if count != 0 {
			return errors.New("already data source exist with the given name")
		}
		if err := tx.Create(datasource).Error; err != nil {
			return err
		}
		// by default we add admin role to the data source.
		internalRole := []*models.Role{}
		internalRole = append(internalRole, &models.Role{
			ObjectID: datasource.ID,
			Name:     "admin",
			Type:     models.DataSourceType,
		})
		dupmap := map[string]struct{}{}
		dupmap["admin"] = struct{}{}
		for _, role := range roles {
			_, ok := dupmap[role]
			if ok {
				continue
			}
			dupmap[role] = struct{}{}
			internalRole = append(internalRole, &models.Role{
				ObjectID: datasource.ID,
				Name:     role,
				Type:     models.DataSourceType,
			})
		}
		return tx.Model(&models.Role{}).Create(&internalRole).Error
	})
}
func (s *Store) GetSessionForUser(userID uint) ([]*models.Session, error) {
	sessions := []*models.Session{}
	if err := s.db.Model(&models.Session{}).Where("user_id = ?", userID).Find(&sessions).Error; err != nil {
		utils.Logger.Error("error while retriving sessions", zap.String("err_msg", err.Error()))
		return nil, err
	}
	return sessions, nil
}

func (s *Store) GetSessionByWhere(query interface{}, args ...interface{}) (*models.Session, error) {
	session := &models.Session{}
	err := s.db.Model(&models.Session{}).Where(query, args...).First(session).Error
	return session, err
}

func (s *Store) GetSessionForAuth(objectID uint, username string, password string) (*models.Session, error) {
	session := &models.Session{}
	err := s.db.Model(&models.Session{}).Where("object_id = ? AND  meta->>'postgresPassword' = ? AND meta->>'postgresUsername' = ?", objectID, password, username).First(session).Error
	return session, err
}

func (s *Store) CreateSessionForUser(userID uint, datasourceID uint) error {
	return s.db.Transaction(func(tx *gorm.DB) error {
		var count int64
		err := tx.Model(&models.Session{}).Where("user_id = ? AND object_id = ?", userID, datasourceID).Count(&count).Error
		if err != nil {
			utils.Logger.Error("error while retriving session count", zap.String("err_msg", err.Error()))
			return err
		}
		if count != 0 {
			return types.ErrSessionExist
		}
		// check whether session already exist for this user.
		session := &models.Session{
			ObjectID: datasourceID,
			UserID:   userID,
			SessionMeta: &models.SessionMeta{
				Type:             "postgres",
				PostgresPassword: utils.GenerateSecureToken(7),
				PostgresUsername: namegenerator.NewNameGenerator(time.Now().UnixNano()).Generate(),
			},
		}
		session.MarshalMeta()
		return tx.Model(&models.Session{}).Create(session).Error
	})
}

func (s *Store) CreateUser(username, password string) (*models.User, error) {
	hashedPassword, err := bcrypt.GenerateFromPassword([]byte(password), bcrypt.DefaultCost)
	if err != nil {
		utils.Logger.Error("error while hashing password", zap.String("err_msg", err.Error()))
		return nil, err
	}
	user := &models.User{
		Name:     username,
		Password: string(hashedPassword),
	}
	if err := s.db.Create(user).Error; err != nil {
		return nil, err
	}
	return user, nil
}

func (s *Store) GetUsers() ([]*models.User, error) {
	users := []*models.User{}
	err := s.db.Model(&models.User{}).Find(&users).Error
	return users, err
}
