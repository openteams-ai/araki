# akari-one

A demo implementation of how akari might be able to work ¯\_(ツ)_/¯

## Contributing

To build and run the project use `pixi`.

### Build an executable
To just build an executable to run akari-one run
```
$ pixi run build
```
Then, a binary will be available in `./target/akari-one`

### Run with pixi
Or, use pixi to run `cargo run`
```
$ pixi run start -- -h
```

## Try it out

Initialize a project
```
$ akari init myproj
```

Activate that environment
```
$ eval "$(akari activate myproj)"
```

From this point, users can use pixi like they normally would. For example, add python and numpy as a dependency to the project.

```
$ pixi add python numpy
```

Deactivate the environment
```
$ eval "$(akari deactivate)"
```

List what other environments are managed by akari by running the `envs` command
```
$ akari envs ls
Available envs:
* myproj
* projmy
```

##  Next steps
* Sort out how saving/checkpointing an environment should work
* Add shell prefix so users know what environment they are in
* Rethink how activation/deactivation of environments should work
