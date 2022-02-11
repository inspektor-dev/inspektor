package systest

import (
	"os/exec"
	"testing"
)

func buildControlPlane(t *testing.T) {
	cmd := exec.Command("go", "build", ".")
	cmd.Dir = "../controlplane"
	err := cmd.Run()
	if err != nil {
		t.Fatal(err)
	}
}

func TestMain(t *testing.T) {
	buildControlPlane(t)
}
