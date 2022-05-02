package store

import (
	"encoding/json"
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
	// check config exist or not.
	_, err := s.Get(types.IntegrationConfigKey)
	if err != nil && err != gorm.ErrRecordNotFound {
		utils.Check(err)
	}
	if err == gorm.ErrRecordNotFound {
		config := &types.IntegrationConfig{}
		val := string(utils.MarshalJSON(config))
		err := s.Set(types.IntegrationConfigKey, val)
		utils.Check(err)
	}
	// check whether admin account exist.
	var count int64
	err = s.db.Model(&models.User{}).Where("name = ?", "admin").Count(&count).Error
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
	if err := s.WriteRoleForObjectID(user.ID, []string{"admin"}, models.UserType); err != nil && err != types.ErrRoleAlreadyExist {
		utils.Logger.Error("error while creating role for the default admin user", zap.String("err_msg", err.Error()))
	}
	return nil
}

func (s *Store) GetUserByName(name string) (*models.User, error) {
	user := &models.User{}
	err := s.db.Model(&models.User{}).Where("name = ?", name).First(user).Error
	return user, err
}

func (s *Store) WriteRoleForObjectID(id uint, roles []string, objectType string) error {
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
			Type:     objectType,
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

func (s *Store) GetDatasource(id uint) (*models.DataSource, error) {
	datasource := &models.DataSource{}
	err := s.db.Model(&models.DataSource{}).Where("id = ?", id).First(datasource).Error
	if err != nil {
		utils.Logger.Error("error while retriving data source", zap.String("err_msg", err.Error()))
		return datasource, err
	}
	return datasource, nil
}

func (s *Store) GetDatasourceByWhere(query interface{}, args ...interface{}) (*models.DataSource, error) {
	dataSource := &models.DataSource{}
	if err := s.db.Model(&models.DataSource{}).Where(query, args...).First(dataSource).Error; err != nil {
		utils.Logger.Error("error while fetching data source", zap.String("err_msg", err.Error()))
		return nil, convertErr(err)
	}
	return dataSource, nil
}

func convertErr(err error) error {
	switch err {
	case gorm.ErrRecordNotFound:
		return types.ErrNotExist
	}
	return err
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

func (s *Store) CreateSessionForUser(userID uint, datasourceID uint, passthrough bool) error {
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
				Passthrough:      passthrough,
			},
		}
		session.MarshalMeta()
		return tx.Model(&models.Session{}).Create(session).Error
	})
}

func (s *Store) CreateSession(sess *models.Session) error {
	return s.db.Transaction(func(tx *gorm.DB) error {
		return tx.Model(&models.Session{}).Create(sess).Error
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

func (s *Store) GetUserByID(id uint) (*models.User, error) {
	user := &models.User{}
	err := s.db.Model(&models.User{}).Where("id = ?", id).First(user).Error
	if err == gorm.ErrRecordNotFound {
		return nil, types.ErrNotExist
	}
	return user, err
}

func (s *Store) UpsertUser(username string, roles []string) (*models.User, error) {
	user, err := s.GetUserByName(username)
	if err != nil {
		if err != gorm.ErrRecordNotFound {
			utils.Logger.Error("error while retriving users", zap.String("err_msg", err.Error()))
			return nil, err
		}
		utils.Logger.Debug("creating user entry since no user exist for the given username", zap.String("username", username))
		user, err := s.CreateUser(username, "ldpusers")
		if err != nil {
			utils.Logger.Error("error while creating user", zap.String("err_msg", err.Error()), zap.String("username", username))
			return nil, err
		}
		err = s.WriteRoleForObjectID(user.ID, roles, models.UserType)
		if err != nil {
			utils.Logger.Error("error while creating roles for the user", zap.String("username", username), zap.String("err_msg", err.Error()))
		}
		return user, err
	}
	utils.Logger.Info("user alreadu exist. syncing roles")
	if len(roles) == 0 {
		return user, nil
	}
	err = s.SyncRoles(user.ID, models.UserType, roles)
	if err != nil {
		return nil, err
	}
	return user, nil
}

func (s *Store) SyncRoles(objectID uint, objectType string, roles []string) error {
	// delete existing roles
	err := s.db.Unscoped().Model(&models.Role{}).Delete(&models.Role{}, "object_id = ? and type = ?", objectID, objectType).Error
	if err != nil {
		utils.Logger.Error("error while deleting existing roles", zap.String("err_msg", err.Error()))
		return err
	}
	return s.WriteRoleForObjectID(objectID, roles, objectType)
}

// GetRoles return unique name of all the roles.
func (s *Store) GetRoles() ([]string, error) {
	results := []*models.Role{}
	err := s.db.Model(&models.Role{}).Distinct("name").Find(&results).Error
	if err != nil {
		return nil, err
	}
	roles := []string{}
	for _, result := range results {
		roles = append(roles, result.Name)
	}
	return roles, nil
}

func (s *Store) GetDataSource() ([]*models.DataSource, error) {
	result := []*models.DataSource{}
	err := s.db.Model(&models.DataSource{}).Find(&result).Error
	return result, err
}

func (s *Store) CreateTempSession(datasourceID uint, roles []string, expiryMinute int64, TempCreatedBy string, ctx json.RawMessage) (*models.Session, error) {
	_, err := s.GetDatasource(datasourceID)
	if err != nil {
		utils.Logger.Error("error while retriving datasoruce", zap.String("err_msg", err.Error()))
		return nil, err
	}
	if len(roles) == 0 {
		return nil, errors.New("atleast one roles is expected to create temp session")
	}
	if expiryMinute == 0 {
		return nil, errors.New("expiry minute should be greater than 0")
	}
	session := &models.Session{
		ObjectID: datasourceID,
		SessionMeta: &models.SessionMeta{
			Type:             "postgres",
			PostgresPassword: utils.GenerateSecureToken(7),
			PostgresUsername: namegenerator.NewNameGenerator(time.Now().UnixNano()).Generate(),
			TempRoles:        roles,
			ExpiresAt:        time.Now().Add(time.Minute * time.Duration(expiryMinute)).UnixNano(),
			TempCreatedBy:    TempCreatedBy,
			Context:          ctx,
		},
	}
	session.MarshalMeta()
	if err := s.CreateSession(session); err != nil {
		utils.Logger.Error("error while creating temp session", zap.String("err_msg", err.Error()))
		return nil, err
	}
	return session, nil
}

func (s *Store) Set(key string, value string) error {
	d := &models.KV{
		Key:   key,
		Value: value,
	}
	return s.db.Model(&models.KV{}).Create(d).Error
}

func (s *Store) Get(key string) (string, error) {
	d := &models.KV{}
	err := s.db.Model(&models.KV{}).Where("key = ?", key).First(d).Error
	return d.Value, err
}

func (s *Store) Update(key string, value string) error {
	return s.db.Model(&models.KV{}).
		Where("key = ?", key).
		Updates(map[string]interface{}{"value": value}).Error
}

func handleGormErr(err error) error {
	if err == nil {
		return nil
	}
	switch err {
	case gorm.ErrRecordNotFound:
		return types.ErrNotExist
	}
	return err
}
