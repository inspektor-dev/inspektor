package cmd

import (
	"fmt"

	"github.com/spf13/cobra"
)

var configFilePath string

func init() {
	rootCmd.PersistentFlags().StringVar(&configFilePath, "config-path", "./config.yaml", "inpektor's config file path")
}

var rootCmd = &cobra.Command{
	Use:   "inspektor",
	Short: "Inspektor helps to secure your data access policy using open policy",
	Run: func(cmd *cobra.Command, args []string) {
		// run you code.
		fmt.Println("to")
	},
}

func Execute() error {
	return rootCmd.Execute()
}
