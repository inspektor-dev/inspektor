---
title: Evaluating open policy agent in rust using wasm
description: Open policy agent is a general purpose policy engine by CNCF. In this tutorial, we'll learn how to evaluate opa polices in rust using wasm
slug: evaluating-open-policy-agent-in-rust-using-wasm
authors:
  - name: Poonai
    title: Maintainer of inspektor
    url: https://twitter.com/poonai_
    image_url: https://i.imgur.com/RNM7R6Q.jpg
tags: [rust,opa,wasm]
image: /img/security.jpg
hide_table_of_contents: false
---
![photo of secuirty gaurd](/img/security.jpg)

OPA (Open Policy Agent) is a policy enforcement engine that can be used for a variety of purposes. OPA's access policies are written in a language called rego. A CNCF-graduated project, it's been incorporated into a number of different products.You can see the list of adopters [here](https://github.com/open-policy-agent/opa/blob/main/ADOPTERS.md).

We chose OPA to enforce database access policies because of its flexibility to write polices as per policy author's need and familiarity in the cloud-native ecosystem.

OPA gives three options to enforce access polices:
- go library 
- rest service
- WASM


The inspektor dataplane is written in rust, so we cannot use the go library to enforce policies in inspektor. For the simplicity, we decided to use WASM to evaluate access policies rather than run a separate rest service. 

Rego policies can be compiled into a wasm module using OPA. The compiled WASM module expose necessary functions to evaluate polices in other language. 

# WASM Compilation

[burrego](https://github.com/kubewarden/policy-evaluator) crate was built by people at `kuberwardern` to evaluate rego policies in rust. In this tutorial, we will learn how to evaluate wasm-compiled rego using the `burrego` crate.

Let's first write a rego programme to evaluate before moving on to the evaluation itself. In the given rego program, set the rule `hello` to  `true` if the given input message `input.message` is `world`.


```rego
package play

default hello = false

hello {
    m := input.message
    m == "world"
}
```

To make use of the policy, run the following command to compile the policy to wasm for the entrypoint `play/hello`

```shell
opa build -t wasm -e play/hello policy.rego
```

The above command will create a bundle.tar.gz file. The tar files contain the following files. 

```
/data.json
/policy.rego
/policy.wasm
/.manifest
```

For this tutorial, we care only about the policy.wasm file, since `policy.wasm` file is the compiled wasm module of rego policy.

# Rust integration

let's add a burrego crate as a dependency to our rust program.

```toml
[dependencies]
burrego = {git = "https://github.com/kubewarden/policy-evaluator"}
```


`Evaluator::new` will take policy as an input and return the `Evaluator` object.

```rust
let policy = fs::read("./policy.wasm").unwrap();
let mut evaluator = Evaluator::new(
        String::from("demo-policy"),
        &policy,
        &DEFAULT_HOST_CALLBACKS,
   ).unwrap();
```

during the evaluation, the entrypoint id is specified to evaluate the entrypoints. Using the `entrypoint_id` function, the id of the entry point can be retrieved. We are retrieving the entrypoint id for `play/hello` in the following snippet.

```rust
let entrypoint_id = evaluator.entrypoint_id(&"play/hello")
```

The policy will be evaluated using `evaluate` function. The `evaluate` function takes entrypoint's id, input and data as paramenter/

```rust
    let input = serde_json::from_str(r#"{"message":"world"}"#).unwrap();
    let data = serde_json::from_str("{}").unwrap();
    let hello = evaluator.evaluate(entrypoint_id, &input, &data).unwrap();
    println!("{}", hello);
```
We got `true` for `play/hello` entrypoint because we passed `message` as `world`. We would have received a `false` result if we had used a different value.
 
```shell
[{"result":true}]
```

I hope you learned how to use evaluate opa policies in rust. Feel free to [join our community in discord](https://t.co/NWnxhxsIx7) where you can follow our development and participate.