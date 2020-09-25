![Test](https://github.com/7mind/binlink/workflows/Test/badge.svg) ![Release](https://github.com/7mind/binlink/workflows/Release/badge.svg)

# binlink

Allows you to link binaries (like `java`) within a directory to expected versions.   

**Warning: this is a work-in-progress project, user experience may be a bit unpleasant for now**

## Why?

1. Other binary dependency managers (`jenv`, `sdkman`) rely on `PATH` mangling and need to be integrated with your shell thus need to be configured separatedly and work differently when you run your scripts/projects different ways.
2. GraalVM overrides some binaries (like `node`) and it's hard to fix that.

`binlink` does not rely on shell-level `PATH` mangling. It maintains a list of symlinks to itself and uses `execve` in order to substitue itself with an appropriate binary. That makes it shell-independent.

## TLDR

```bash
cp binlink /usr/local/bin
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
