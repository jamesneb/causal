package discoverycmd
import (
	"fmt"
	"github.com/spf13/cobra"

	awscmd "discovery.com/m/v2/aws"
	"discovery.com/m/v2/identity"
)
type region int

const (
	ALL region = iota
	USEAST1 
	TOTALREGIONS // This must always be the last value in the const block
)



var SelectedRegion string
var RoleArn string
var SessionName string = "discovery-cli-session"

// When we add additional providers we will add an additional flag
var listCmd = &cobra.Command{
	Use: "list [region] [roleArn]",
	Short: "Discover and list services",
	Long: "Discover and list services running on various platforms.",
	Args: cobra.ExactArgs(2),
	Run: func(cmd *cobra.Command, args []string) {
		SelectedRegion = args[0]
		RoleArn = args[1]
		
		// Authenticate with Auth0
		auth0Config, err := identity.NewAuth0Config()
		if err != nil {
			fmt.Printf("Error creating Auth0 config: %v\n", err)
			return
		}
		
		err = auth0Config.Login()
		if err != nil {
			fmt.Printf("Error authenticating with Auth0: %v\n", err)
			return
		}
		
		if auth0Config.Token == nil {
			fmt.Println("Authentication failed: No token received")
			return
		}
		
		// Use the token's ID token for AWS role assumption
		idToken := auth0Config.Token.AccessToken
		
		// Set the ID token for AWS operations
		SessionName = "discovery-cli-session"
		
		HandleRegionArgument(idToken)
	},
}

func GetListCmd() *cobra.Command {

	return listCmd

}

// Begin manual instrumentation 

func HandleRegionArgument(idToken string) {
		switch (SelectedRegion) {
			case "ALL": BuildAllRegions(idToken)
			case "US-EAST-1": BuildRegion(USEAST1, idToken)
			default:
				fmt.Printf("Unsupported region: %s\n", SelectedRegion)
	}
}

// TODO: Fill out with all AWS regions
func BuildRegion(r region, idToken string) {
	region_string := ""
	
	switch (r) {
		case 1: region_string = "us-east-1"
	}

	fmt.Printf("Discovering services in region %s with role %s\n", region_string, RoleArn)
	err := awscmd.CatalogServices(region_string, RoleArn, idToken, SessionName)	
	if err != nil {
		fmt.Printf("Error cataloging services: %v\n", err)
	}
}

func BuildAllRegions(idToken string) {
	fmt.Println("Discovering services in all regions...")
	for i := 1; i< int(TOTALREGIONS); i++ {
		BuildRegion(region(i), idToken)		
	}
}

