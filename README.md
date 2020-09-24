# binlink

Allows you to link binaries (like `java`) within a directory to expected versions.   

**Warning: this is a work-in-progress project**

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