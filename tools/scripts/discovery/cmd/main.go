package main
import "discovery.com/m/v2/cmd/discoverycmd"

func main() {
	
	discoverycmd.RootCmd.AddCommand(discoverycmd.GetListCmd())
	
	discoverycmd.Execute()

	discoverycmd.HandleRegionArgument()
}

