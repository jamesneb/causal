package discoverycmd
import (
	"github.com/spf13/cobra"
)

// When we add additional providers we'll add an additional flag
var SelectedRegion string
var listCmd = &cobra.Command{
	Use: "list",
	Short: "Discover and list services",
  Long: "Discover and list services running on various platforms. ",
	ValidArgs: []string{"ALL", "US-EAST-1", "US-EAST-2", "US-WEST-1", "US-WEST-2"},
	Args: cobra.MatchAll(cobra.ExactArgs(1),  cobra.OnlyValidArgs),
  Run: func(cmd *cobra.Command, regions []string) {
			SelectedRegion = regions[0]

},
}

func getListCmd() *cobra.Command {

	return listCmd

}
