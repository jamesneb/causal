package aws

import (
	"context"
	"fmt"
	"time"
	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/aws/aws-sdk-go-v2/config"
	"github.com/aws/aws-sdk-go-v2/service/iam"
	"github.com/aws/aws-sdk-go-v2/service/sts"
	stscreds "github.com/aws/aws-sdk-go-v2/credentials/stscreds"
)

func setupBaseConfig() (aws.Config, error) {
	
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

func createIAMClient(cfg aws.Config) (*iam.Client) {

	// Create IAM Client

	client := iam.NewFromConfig(cfg)
	return client 
}

func createSTSClient(cfg aws.Config) (*sts.Client) {
	client := sts.NewFromConfig(cfg)
	return client 
}

func createRoleCredentials(stsClient sts.Client, roleArn string) *stscreds.AssumeRoleProvider {

	roleCredentials := stscreds.NewAssumeRoleProvider(&stsClient, roleArn, func(o *stscreds.AssumeRoleOptions) {

	o.RoleSessionName = fmt.Sprintf("Probe Session-%s", time.Now().Format("20060102T150405"))
	o.Duration = time.Hour
	})
	return roleCredentials
}

func createIAMConfig(roleCredentials *stscreds.AssumeRoleProvider, baseCfg aws.Config ) (aws.Config) {


	// New AWS config with assumed role 

	assumedCfg := aws.Config{
		Region: baseCfg.Region,
		Credentials: aws.NewCredentialsCache(roleCredentials),
	}

	return assumedCfg

}
