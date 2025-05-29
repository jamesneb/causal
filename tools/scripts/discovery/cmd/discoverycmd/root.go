package discoverycmd
import (
	"os"
	"log"
	"github.com/spf13/cobra"
)

var RootCmd = &cobra.Command{
	Use: "Discovery",
	Short: "Service discovery CLI",
	Long: "Finds services inside cloud platforms" ,
}


func Execute() {
	if err := RootCmd.Execute(); err != nil {
		log.Fatal(err)
		os.Exit(1)
	}
}
