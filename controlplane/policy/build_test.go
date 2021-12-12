package policy

import (
	"testing"
)

func TestBuild(t *testing.T) {
	_, err := Build("sdf")
	if err != nil {
		t.Fatal(err)
	}
	//ioutil.WriteFile("policy.wasm", buf, 0644)
}
