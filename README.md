![Test](https://github.com/7mind/binlink/workflows/Test/badge.svg) ![Release](https://github.com/7mind/binlink/workflows/Release/badge.svg)

# binlink

Allows to create per-directory overrides for your toolchain binaries of expected versions. E.g. you may say that `java` should point to `AdoptOpenJDK 11` when invoked in directory `~/work/my-project`.

**Warning: this is a work-in-progress project, user experience may be a bit unpleasant for now**

## Why?

1. Other binary dependency managers (`jenv`, `sdkman`) rely on `PATH` mangling and need to be integrated with your shell thus need to be configured separatedly and work differently when you run your scripts/projects different ways.
2. GraalVM overrides some binaries (like `node`) and it's hard to fix that.

`binlink` does not rely on shell-level `PATH` mangling. It maintains a list of symlinks to itself and uses `execve` in order to substitue itself with an appropriate binary. That makes it shell-independent.

## TLDR

```bash
brew tap 7mind/tools
brew install binlink

binlink link
echo 'PATH=/usr/local/bin/binlinks/:${PATH}' >> ~/.zshrc
```

then

```bash
cd ~/work/project
binlink example > .binlink.toml
nano .binlink.toml 
```

default config path is `~/.config/binlink/binlink.toml`. 

Local config would always work as overlay.

## See also

- https://github.com/shyiko/jabba
- https://github.com/jenv/jenv
- https://github.com/sdkman/sdkman-cli
