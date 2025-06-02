package main

import (
	"fmt"
	"os"
	
	"discovery.com/m/v2/cmd/discoverycmd"
)

func main() {
	fmt.Println("Discovery CLI - Service Discovery Tool")
	
	// Add list command to root command
	discoverycmd.RootCmd.AddCommand(discoverycmd.GetListCmd())
	
	// Execute the root command
	if err := discoverycmd.RootCmd.Execute(); err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
}

