package cmd

import (
	"fmt"
	"os"

	"inspektor/config"
	"inspektor/utils"

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
		fmt.Println("to")
	},
}

func Execute() error {
	return rootCmd.Execute()
}
