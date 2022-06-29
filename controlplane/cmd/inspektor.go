package cmd

import (
	"log"
	"net/http"
	"os"

	"inspektor/config"
	"inspektor/handlers"
	"inspektor/metrics"
	"inspektor/models"
	"inspektor/policy"
	"inspektor/rpcserver"
	"inspektor/slackbot"
	"inspektor/store"
	"inspektor/teamsbot"
	"inspektor/utils"

	"github.com/gorilla/mux"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
	"go.uber.org/zap"
)

var configFilePath string

func init() {
	rootCmd.PersistentFlags().StringVar(&configFilePath, "config-path", "./config.yaml", "inpektor's config file path")
}

var rootCmd = &cobra.Command{
	Use:   "inspektor",
	Short: "Inspektor helps to secure your data access policy using open policy",
	Run: func(cmd *cobra.Command, args []string) {
		// set the path for yaml config file.
		viper.SetConfigType("yaml")
		viper.SetConfigFile(configFilePath)
		config := &config.Config{}
		if err := viper.ReadInConfig(); err != nil {
			if os.IsNotExist(err) {
				utils.Logger.Fatal("config file is missing", zap.String("config_file_path", configFilePath))
			}
			utils.Logger.Fatal("error while reading config file", zap.String("err_msg", err.Error()))
		}
		viper.Unmarshal(config)
		if err := config.Validate(); err != nil {
			utils.Logger.Fatal("error while validating config file", zap.String("err_msg", err.Error()))
		}
		db, err := utils.GetDB(config)
		if err != nil {
			utils.Logger.Fatal("error while connecting with postgres database", zap.String("err_msg", err.Error()))
		}
		models.Migrate(db)
		store, err := store.NewStore(db)
		if err != nil {
			utils.Logger.Fatal("error while creating store interface", zap.String("err_msg", err.Error()))
		}
		policyManager := policy.NewPolicyManager(config)
		if err := policyManager.Init(); err != nil {
			utils.Logger.Fatal("error while initializing policy manager", zap.String("err_msg", err.Error()))
		}
		var metricsHandler *metrics.MetricsHandler
		// start the slack bot if the config given
		if config.SlackBotToken != "" {
			utils.Logger.Info("starting slack bot")
			bot := slackbot.New(config, store)
			go bot.Start()
			metricsHandler = metrics.NewMetricsHandler(bot)
			go metricsHandler.Start()
		}
		server := rpcserver.NewServer(store, policyManager, metricsHandler)
		go func(server *rpcserver.RpcServer) {
			if err := server.Start(config); err != nil {
				log.Fatal(err)
			}
		}(server)
		teamsBotHandler, err := teamsbot.New("temp", "temp")
		if err != nil {
			utils.Logger.Fatal("error while intializing teams bot handler", zap.String("err_msg", err.Error()))
		}
		h := handlers.Handlers{
			Store:        store,
			Cfg:          config,
			Policy:       policyManager,
			TeamsHandler: teamsBotHandler.HandleTeamsNotification,
		}
		router := mux.NewRouter()
		h.Init(router)
		utils.Logger.Info("starting control plane", zap.String("listen_port", config.ListenPort))
		log.Fatal(http.ListenAndServe(config.ListenPort, router))
	},
}

func Execute() error {
	return rootCmd.Execute()
}
