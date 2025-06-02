package identity
import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"net/url"
	"time"
	"golang.org/x/oauth2"
	"github.com/coreos/go-oidc/v3/oidc"
)

type Auth0Config struct {

	Domain	string
	ClientID	string
	Audience	string
	Token	*oauth2.Token
	Verifier	*oidc.IDTokenVerifier

}

func NewAuth0Config() (*Auth0Config, error) {
	
	domain :=  "OAUTH DOMAIN"
	clientID := "AUTH0_CLIENT_ID"
	audience := ""


	provider, err := oidc.NewProvider(context.Background(), "https://"+domain+"/")
	if err != nil {
		return nil, fmt.Errorf("Failed to get provider: %w", err) 
	}
	
	return &Auth0Config{
		Domain: domain,
		ClientID: clientID,
		Audience: audience,
		Verifier: provider.Verifier(&oidc.Config{ClientID: clientID}),
	}, nil
}

func (cfg *Auth0Config) Login() error {

	deviceEndpoint := fmt.Sprintf("https://%s/oauth/device/code", cfg.Domain)
	tokenEndpoint := fmt.Sprintf("https://%s/oauth/token", cfg.Domain)
	
	data := url.Values{}
	data.Set("client_id", cfg.ClientID)
	data.Set("scope", "openid profile email")
	
	if cfg.Audience != "" {
		data.Set("audience", cfg.Audience)
	}
	
	resp, err := http.PostForm(deviceEndpoint, data)
	
	if err != nil {
		return fmt.Errorf("failed to request device code: %w", err)
	}
	
	defer resp.Body.Close()

	var deviceResp struct {

		DeviceCode	string `json:"device_code"`
		UserCode    string `json:"user_code"`
		VerificationURI string `json:"verification_uri"`
		VerificationURIComplete string `json:"verification_uri_complete"`
		Interval	int	`json:"interval"`		
	}
	json.NewDecoder(resp.Body).Decode(&deviceResp)

	fmt.Printf("Please open this url in your browser:\n\n%s\n\n", deviceResp.VerificationURIComplete)

	for {
		time.Sleep(time.Duration(deviceResp.Interval) * time.Second)

		form := url.Values{}
		form.Set("grant_type", "urn:ietf:params:oauth:grant_type:device_code")
		form.Set("device_code", deviceResp.DeviceCode)
		form.Set("client_id", cfg.ClientID)

		tokResp, err := http.PostForm(tokenEndpoint, form)
		if err != nil {
			return err
		}
		defer tokResp.Body.Close()
		
		var tokenData map[string]interface{}
		json.NewDecoder(tokResp.Body).Decode(&tokenData)

		if tokResp.StatusCode == http.StatusOK {
			
			tokBytes, _ := json.Marshal(tokenData)
			token := &oauth2.Token{}
			json.Unmarshal(tokBytes, token)
			
			cfg.Token = token 
			return nil
		
		} 

		if tokenData["error"] != nil && tokenData["error"] != "authorization_pending" {
		
			return fmt.Errorf("login error: %v", tokenData["error"])
		}


		}

		
}


