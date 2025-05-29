package discovery
import (
	"fmt"
	"github.com/aws/aws-sdk-go-v2/aws"
  "github.com/aws/aws-sdk-go-v2/config"
  "github.com/aws/aws-sdk-go-v2/service/dynamodb"
	"context"
	"log"
)

type provider int 
const ( 
	AWS provider = iota
)

	
func GetServicesByProvider(name provider) {
	switch name {
	case AWS: 
		cfg, err := config.LoadDefaultConfig(context.TODO(), config.WithRegion("us-west-2"))    
		if err != nil { 
			log.Fatalf("unable to load SDK config, %v", err)
		}	

	}
}


