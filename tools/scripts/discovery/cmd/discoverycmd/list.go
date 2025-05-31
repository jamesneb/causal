package discoverycmd
import (
	"github.com/spf13/cobra"
)
type region int
const (
	ALL region = iota
	USEAST1 
	TOTALREGIONS // This must always be the last value in the const block
)
var SelectedRegion string

// When we add additional providers we will add an additional flag
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

func GetListCmd() *cobra.Command {

	return listCmd

}

// Begin manual instrumentation 

func HandleSelectedRegion() {

		switch (SelectedRegion) {
			case "ALL": BuildAllRegions()
			default: BuildRegion(SelectedRegion)
			
			
	}
}

func BuildAllRegions() {

	for (int i = 0; i < TOTALREGIONS; i++) {
		
		ok, err :=	AssumeIAMRole(i)
		if err != nil {
			fmt.Println("Error:", err) 	
		}
		

	}

}

func AssumeIAMRole(region int) {



}
