# ubuild

The ubuild, standing for universal build, builds softwares in the universal
environment.

## How to use

```shell
# ubuild <IMAGE> <COMMAND>
$ ubuild rund:master make LIBC=gnu
```

Note that DO NOT use ubuild with sudo. Otherwise, the default base path
will be the root's home directory. By default, the ubuild starts the
container with sudo. If you don't want this, a `--no-sudo` argument should
be applied.

Please refer to `ubuild -h` for more usages.
