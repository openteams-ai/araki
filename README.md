# araki

A demo implementation of how araki might be able to work ¯\_(ツ)_/¯

## Contributing

To build and run the project use `pixi`.

### Build an executable
To just build an executable to run araki run
```
$ pixi run build
```
Then, a binary will be available in `./target/araki`

### Run with pixi
Or, use pixi to run `cargo run`
```
$ pixi run start -- -h
```

## Try it out

Initialize a project
```
$ araki init myproj
```

Activate that environment
```
$ eval "$(araki activate myproj)"
```

From this point, users can use pixi like they normally would. For example, add python and numpy as a dependency to the project.

```
$ pixi add python=3.13 numpy=2.3
```

Save a checkpoint by running the `tag` command
```
$ araki tag v1 --description "python 3.13 and numpy 2.3"
```

List available tags
```
$ araki list
```

Checkout the latest tag (determined from the git tree) of an environment
```
$ araki checkout latest
```

Deactivate the environment
```
$ eval "$(araki deactivate)"
```

List what other environments are managed by araki by running the `envs` command
```
$ araki envs ls
Available envs:
* myproj
* projmy
```

### Use a remote source
Initialize a project with a remote backend (must use ssh url and have your ssh key loaded into your keychain)
```
$ araki init abc --source git@github.com:soapy1/test-abc.git

```
Push/pull from a remote source
```
$ araki pull
```

```
$ araki push v1
```

##  Next steps
* Rethink how activation/deactivation of environments should work
