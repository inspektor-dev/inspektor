package policy

import (
	"archive/tar"
	"bytes"
	"compress/gzip"
	"context"
	"errors"
	"io"

	"github.com/open-policy-agent/opa/ast"
	"github.com/open-policy-agent/opa/compile"
)

func Build(path string) ([]byte, error) {
	// compile the given policy to wasm target.
	out := &bytes.Buffer{}
	compiler := compile.New().
		WithCapabilities(ast.CapabilitiesForThisVersion()).
		WithTarget(compile.TargetWasm).
		WithEntrypoints("inspektor/resource_acl/allow").
		WithOutput(out).
		WithPaths("/home/poonai/inspektor/controlplane/policy/testpolicy")
	err := compiler.Build(context.TODO())
	if err != nil {
		return nil, err
	}
	// retrive the wasm binary from tar output
	reader, err := gzip.NewReader(out)
	if err != nil {
		return nil, err
	}
	tarReader := tar.NewReader(reader)
	for {
		header, err := tarReader.Next()
		if err != nil {
			return nil, err
		}
		switch header.Typeflag {
		case tar.TypeReg:
			// check whether the file name is policy.wasm
			if header.Name != "/policy.wasm" {
				continue
			}
			// read the policy file and return it.
			policy := &bytes.Buffer{}
			size, err := io.Copy(policy, tarReader)
			if err != nil {
				return nil, err
			}
			if size != header.Size {
				return nil, errors.New("unable to read the policy")
			}
			return policy.Bytes(), err
		}
	}
}
