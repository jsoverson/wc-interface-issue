# wasmcloud interface issue

wapc-generated rust doesn't produce usable compatible interfaces across actors and providers (as they are defined now).

This is a problem that could be fixed in wapc-codegen, actor-interfaces, or capability providers. It feels like wapc-codegen is generating un-intuitive interface signatures.
  
E.g. with blobstore:

ListObjects is defined as taking one argument, `container` of type `Container`
https://github.com/wasmCloud/actor-interfaces/blob/main/blobstore/blobstore.widl#L7

The `list_objects` function is generated with the same signature https://github.com/wasmCloud/actor-interfaces/blob/codegen_roles/blobstore/rust/src/generated.rs#L85

but the function serializes those arguments into a `ListObjectArgs` https://github.com/wasmCloud/actor-interfaces/blob/codegen_roles/blobstore/rust/src/generated.rs#L86-L92

`ListObjectsArgs` is a struct that contains a `container` field and so serializes to a structure like `{"container":{"id":"blah"}}`

In both `fs` and `s3` providers, `list_objects` is implemented to expect a `Container`, not a `ListObjectsArgs` https://github.com/wasmCloud/capability-providers/blob/main/fs/src/lib.rs#L144-L148

Any actor that uses an interface like this dies with a `[trap] unreachable executed`. The call outputs nothing useful by default, on either the call side or wasmcloud side.

FWIW: changing the widl to use curly braces vs parens fixes this (e.g. `ListObjects{container: Container}: BlobList`). It avoids the `[Method]Args` type abstraction. It's not clear if this is expected behavior or a coincidence.

## Steps to reproduce

### Clone example repo

```
git clone https://github.com/jsoverson/wc-interface-issue && cd wc-interface-issue
```

### Ensure wapc is up-to-date

```zsh
npm install -g https://github.com/wapc/cli.git
```

### build actor

```
wapc generate wapc.yaml
cargo build
wash claims sign target/wasm32-unknown-unknown/debug/test.wasm --name=test --ver=1 --rev=0
```

### run wasmcloud

```zsh
export ACTOR=$(wash claims inspect ./target/wasm32-unknown-unknown/debug/test_s.wasm -o json | jq -r '.module')
RUST_LOG=trace wasmcloud -m manifest.yaml
```

### call actor

```zsh
export ACTOR=$(wash claims inspect ./target/wasm32-unknown-unknown/debug/test_s.wasm -o json | jq -r '.module')
wash ctl call $ACTOR listdir '"tmp"'
```

## Expected

Get a list of files in `/tmp` directory

## Actual 1

First error is an unintuitive serialization error.

```
⠀Calling actor MDWGV7FTDUP4M6CI6UZVLKNQGQM67HILOK2M5UJ6MG57CGZNKQTOUCCJ ... 
  Error invoking actor: Failed to invoke actor: Guest call failure: Guest call failed: Failed to de-serialize: invalid type: string "target", expected struct ListdirArgs
```

## Actual 2

The first attempt at fixing this problem on the actor's side is to change the data passed to the `listdir` operation from a string to a `ListDirArgs` structure:

```zsh
export ACTOR=$(wash claims inspect ./target/wasm32-unknown-unknown/debug/test_s.wasm -o json | jq -r '.module')
wash ctl call $ACTOR listdir '{"dir":"tmp"}'
```

But that then produces a `Guest call failure: [trap] unreachable executed` error because the provider dies at deserialization. There's no output on the wasmcloud side, even with trace logging on.

```
Calling actor MDWGV7FTDUP4M6CI6UZVLKNQGQM67HILOK2M5UJ6MG57CGZNKQTOUCCJ ... 
Error invoking actor: Failed to invoke actor: Guest call failure: [trap] unreachable executed
```
