package aws

import (
	"context"
	"encoding/base32"
	"fmt"
	"net/http"
	"sync"
	"time"

	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/aws/aws-sdk-go-v2/config"
	stscreds "github.com/aws/aws-sdk-go-v2/credentials/stscreds"
	"github.com/aws/aws-sdk-go-v2/service/iam"
	"github.com/aws/aws-sdk-go-v2/service/lambda"
	"github.com/aws/aws-sdk-go-v2/service/sts"
)

type Service struct {
	ServiceName  string
	Configuration map[string]string
	Code         map[string]string
	Concurrency  map[string]string
	Tags         map[string]string
}

var ServicePool = sync.Pool{
		New: func() any {
				 return &Service{}
	},
}

func GetService() *Service {

	return ServicePool.Get().(*Service) 

}

func PutService(s *Service) {
	s.ServiceName = ""
	s.Configuration = nil
	s.Code = nil
	s.Concurrency = nil
	s.Tags = nil
	ServicePool.Put(s)
}

func SetupBaseConfig() (aws.Config, error) {
	
	ctx := context.TODO()

	// Load default config, typically from instance metadata service. 
	// This will be used to load a permanent IAM role
	
	cfg, err := config.LoadDefaultConfig(ctx)
	
	if err != nil {
	
		fmt.Printf("Unable to load AWS SDK config file, %v", err)
		return aws.Config{}, err

	}

	return cfg, nil
}

func CreateIAMClient(cfg aws.Config) *iam.Client {

	// Create IAM Client

	client := iam.NewFromConfig(cfg)
	return client 
}

func CreateSTSClient(cfg aws.Config) *sts.Client {
	client := sts.NewFromConfig(cfg)
	return client 
}

func CreateRoleCredentials(stsClient sts.Client, roleArn string) *stscreds.AssumeRoleProvider {

	roleCredentials := stscreds.NewAssumeRoleProvider(&stsClient, roleArn, func(o *stscreds.AssumeRoleOptions) {

	o.RoleSessionName = fmt.Sprintf("Probe Session-%s", time.Now().Format("20060102T150405"))
	o.Duration = time.Hour
	})
	return roleCredentials
}

func CreateIAMConfig(roleCredentials *stscreds.AssumeRoleProvider, baseCfg aws.Config, region string) aws.Config {


	// New AWS config with assumed role 

	assumedCfg := aws.Config{
		Region: region,
		Credentials: aws.NewCredentialsCache(roleCredentials),
	}

	return assumedCfg

}

// TODO: Refactor the source code download logic into separate method so that we can handle 
// errors

func CatalogLambdas(cfg aws.Config) {

	ctx := context.TODO()
	lambdaClient := lambda.NewFromConfig(cfg)
	
	paginator := lambda.NewListFunctionsPaginator(lambdaClient, &lambda.ListFunctionsInput{})

	for paginator.HasMorePages() {
		page, err := paginator.NextPage(ctx)
		if err != nil {
    	fmt.Printf("Error getting page: %v", err)
		}

		for _, fn := range page.Functions {
			

			output, err := lambdaClient.GetFunction(ctx, &lambda.GetFunctionInput{
			FunctionName: fn.FunctionName,
			
			})
			if err != nil {
				fmt.Printf("Failed to get function info: %v", err)
			}
			
			service := GetService()
			service.ServiceName = *fn.FunctionName
			
			// Convert AWS types to string maps
			if output.Configuration != nil {
				service.Configuration = make(map[string]string)
				if output.Configuration.FunctionName != nil {
					service.Configuration["FunctionName"] = *output.Configuration.FunctionName
				}
				
				if output.Configuration.Runtime != "" {
					service.Configuration["Runtime"] = string(output.Configuration.Runtime)
				}
				if output.Configuration.Role != nil {
					service.Configuration["Role"] = *output.Configuration.Role
				}
				if output.Configuration.Handler != nil {
					service.Configuration["Handler"] = *output.Configuration.Handler
				}
				if output.Configuration.Description != nil {
					service.Configuration["Description"] = *output.Configuration.Description
				}
			}
		  	
			if output.Code != nil {
				service.Code = make(map[string]string)
				if output.Code.Location != nil {
					service.Code["Location"] = *output.Code.Location
				}
				if output.Code.RepositoryType != nil {
					service.Code["RepositoryType"] = *output.Code.RepositoryType
				}
			}
			
			if output.Concurrency != nil {
				service.Concurrency = make(map[string]string)
				if output.Concurrency.ReservedConcurrentExecutions != nil {
					service.Concurrency["ReservedConcurrentExecutions"] = fmt.Sprintf("%d", *output.Concurrency.ReservedConcurrentExecutions)
				}
			}
			
			if output.Tags != nil {
				service.Tags = make(map[string]string)
				for k, v := range output.Tags {
					service.Tags[k] = v
				}
			}
			
			// Return service to pool when done
			PutService(service)
		}

	}
}
	


func CatalogServices(region string, roleArn string) {


	
	baseCfg, err := SetupBaseConfig()

	if err != nil {
		
		fmt.Printf("failed to load config, %v", err)
		return
	}

	stsClient := CreateSTSClient(baseCfg)
  
	roleProvider := CreateRoleCredentials(stsClient, roleArn)

	cfg := CreateIAMConfig(roleProvider, baseCfg, region)
	
	// LAMBDA

	CatalogLambdas(cfg)
}
