package discovery
import "fmt"

type provider int 
const ( 
	AWS provider = iota
)

	
func ConnectTo(name provider) {
	switch name {
		case AWS: return 
	}
}
